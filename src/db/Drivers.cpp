#include "Drivers.h"
#include "../core/ConnectionStore.h"

#include <QSqlDatabase>
#include <QSqlError>
#include <QSqlQuery>
#include <QSqlRecord>
#include <QThread>
#include <QTcpSocket>
#include <QElapsedTimer>

namespace {
// 生成一个唯一连接名（用于 Qt SQL 多连接场景）
QString makeConnName(const QString& base) {
    return QStringLiteral("_sqler_%1_%2").arg(base).arg(reinterpret_cast<qulonglong>(QThread::currentThread()));
}

// 简单 RAII 封装：确保 QSqlDatabase 临时连接被清理
class ScopedSqlConnection {
public:
    explicit ScopedSqlConnection(QSqlDatabase db) : m_db(std::move(db)) {}
    ~ScopedSqlConnection() { close(); }
    QSqlDatabase& db() { return m_db; }
    void close() {
        if (!m_db.isValid()) return;
        const QString name = m_db.connectionName();
        QSqlDatabase tmp = m_db; // detach
        tmp.close();
        {
            QSqlDatabase nullDb;
            tmp = nullDb;
        }
        QSqlDatabase::removeDatabase(name);
        m_db = QSqlDatabase();
    }
private:
    QSqlDatabase m_db;
};

// MySQL 驱动实现
class MySqlDriver final : public IDbDriver {
public:
    bool testConnection(const ConnectionConfig& cfg, QString& error) override {
        auto res = open(cfg, error);
        if (!res) return false;
        // close by RAII
        return true;
    }

    QStringList listTables(const ConnectionConfig& cfg, QString& error) override {
        auto scoped = open(cfg, error);
        if (!scoped) return {};
        return scoped->db().tables();
    }

    QueryResult executeSql(const ConnectionConfig& cfg, const QString& sql) override {
        QueryResult r;
        QString err;
        auto scoped = open(cfg, err);
        if (!scoped) { r.ok = false; r.error = err; return r; }
        QSqlQuery q(scoped->db());
        if (!q.exec(sql)) {
            r.ok = false;
            r.error = q.lastError().text();
            return r;
        }
        // 如果有记录集，收集列与行
        if (q.isSelect()) {
            const auto rec = q.record();
            for (int i = 0; i < rec.count(); ++i) r.columns.push_back(rec.fieldName(i));
            while (q.next()) {
                QVariantList row;
                row.reserve(rec.count());
                for (int i = 0; i < rec.count(); ++i) row.push_back(q.value(i));
                r.rows.push_back(std::move(row));
            }
        } else {
            r.affected = q.numRowsAffected();
        }
        r.ok = true;
        return r;
    }

private:
    static std::optional<ScopedSqlConnection> open(const ConnectionConfig& cfg, QString& error) {
        const QString driver = QStringLiteral("QMYSQL");
        if (!QSqlDatabase::isDriverAvailable(driver)) {
            error = QStringLiteral("未找到 MySQL 驱动(QMYSQL)");
            return std::nullopt;
        }
        const QString connName = makeConnName(cfg.name.isEmpty() ? cfg.id : cfg.name);
        QSqlDatabase db = QSqlDatabase::addDatabase(driver, connName);
        db.setHostName(cfg.host);
        if (cfg.port > 0) db.setPort(cfg.port);
        db.setUserName(cfg.user);
        db.setPassword(cfg.password);
        if (!cfg.database.isEmpty()) db.setDatabaseName(cfg.database);
        if (!db.open()) {
            error = db.lastError().text();
            QSqlDatabase::removeDatabase(connName);
            return std::nullopt;
        }
        return ScopedSqlConnection(std::move(db));
    }
};

// Redis 驱动实现（仅测试连接：AUTH -> PING -> SELECT dbIndex）
// 说明：使用最简 RESP 文本协议，不依赖第三方库
class RedisDriver final : public IDbDriver {
public:
    bool testConnection(const ConnectionConfig& cfg, QString& error) override {
        QTcpSocket sock;
        sock.connectToHost(cfg.host.isEmpty() ? QStringLiteral("127.0.0.1") : cfg.host,
                           cfg.port > 0 ? cfg.port : 6379);
        if (!sock.waitForConnected(3000)) { error = sock.errorString(); return false; }

        // AUTH（可选）
        if (!cfg.password.isEmpty()) {
            if (!sendAuth(sock, cfg.user, cfg.password, error)) return false;
        }

        // SELECT db（若 database 是数字）
        bool okNum = false;
        int dbIndex = cfg.database.toInt(&okNum);
        if (okNum) {
            if (!sendCommandExpectSimpleString(sock, QStringList{QStringLiteral("SELECT"), QString::number(dbIndex)}, QStringLiteral("OK"), error))
                return false;
        }

        // PING
        if (!sendCommandExpectSimpleString(sock, {QStringLiteral("PING")}, QStringLiteral("PONG"), error)) return false;
        return true;
    }

private:
    static bool sendAuth(QTcpSocket& s, const QString& user, const QString& pass, QString& error) {
        QStringList args;
        args << QStringLiteral("AUTH");
        if (!user.isEmpty()) args << user << pass; else args << pass;
        return sendCommandExpectSimpleString(s, args, QStringLiteral("OK"), error);
    }

    // 发送 RESP 简单命令并期望简单字符串回复（+OK 或 +PONG）
    static bool sendCommandExpectSimpleString(QTcpSocket& s, const QStringList& args, const QString& expect, QString& error) {
        QByteArray payload = buildRespArray(args);
        if (s.write(payload) == -1 || !s.waitForBytesWritten(2000)) { error = s.errorString(); return false; }
        if (!s.waitForReadyRead(3000)) { error = QStringLiteral("等待 Redis 响应超时"); return false; }
        const QByteArray resp = s.readAll();
        if (resp.isEmpty()) { error = QStringLiteral("Redis 返回空响应"); return false; }
        if (resp[0] == '-') { // 错误
            error = QString::fromUtf8(resp.mid(1)).trimmed();
            return false;
        }
        if (resp[0] == '+') {
            const QString msg = QString::fromUtf8(resp.mid(1)).trimmed();
            if (msg.compare(expect, Qt::CaseInsensitive) == 0) return true;
            error = QStringLiteral("Redis 返回: %1").arg(msg);
            return false;
        }
        // 其他类型简单忽略解析，按失败处理
        error = QStringLiteral("未预期的 Redis 响应类型");
        return false;
    }

    static QByteArray buildRespBulk(const QByteArray& s) {
        QByteArray out;
        out += "$" + QByteArray::number(s.size()) + "\r\n" + s + "\r\n";
        return out;
    }

    static QByteArray buildRespArray(const QStringList& parts) {
        QByteArray out;
        out += "*" + QByteArray::number(parts.size()) + "\r\n";
        for (const auto& p : parts) out += buildRespBulk(p.toUtf8());
        return out;
    }
};
} // namespace

std::unique_ptr<IDbDriver> createDriverFor(const QString& type) {
    const QString t = type.trimmed().toLower();
    if (t == QLatin1String("mysql")) return std::make_unique<MySqlDriver>();
    if (t == QLatin1String("redis")) return std::make_unique<RedisDriver>();
    return {}; // 未知类型
}

