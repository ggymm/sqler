// 连接配置存储：负责把连接信息（ConnectionConfig）持久化到 JSON 文件，
// 并提供加载/保存/查询/更新等接口。
#include "ConnectionStore.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QStandardPaths>
#include <QUuid>
#include "Config.h"

namespace {
// 将 ConnectionConfig 序列化为 QJsonObject
static QJsonObject toJson(const ConnectionConfig& c) {
    QJsonObject o;
    o["id"] = c.id;
    o["name"] = c.name;
    o["type"] = c.type;
    o["host"] = c.host;
    o["port"] = c.port;
    o["user"] = c.user;
    o["password"] = c.password; // NOTE: plain text for now
    o["database"] = c.database;
    return o;
}

// 反序列化：QJsonObject -> ConnectionConfig
static ConnectionConfig fromJson(const QJsonObject& o) {
    ConnectionConfig c;
    c.id = o.value("id").toString();
    c.name = o.value("name").toString();
    c.type = o.value("type").toString();
    c.host = o.value("host").toString();
    c.port = o.value("port").toInt();
    c.user = o.value("user").toString();
    c.password = o.value("password").toString();
    c.database = o.value("database").toString();
    return c;
}
}

// 单例实例
ConnectionStore& ConnectionStore::instance() {
    static ConnectionStore s;
    return s;
}

// 构造时尝试加载已有连接
ConnectionStore::ConnectionStore(QObject* parent)
    : QObject(parent) {
    load();
}

// 首选用户自定义的数据目录，其次采用平台 AppData 目录
QString ConnectionStore::dataDir() const {
    // Prefer user-configured data dir, else default to platform location
    QString configured = Config::instance().get().dataDir;
    if (!configured.isEmpty()) return configured;
    const auto path = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation);
    return path.isEmpty() ? QDir::homePath() + "/.sqler" : path;
}

// 连接文件路径（connections.json）
QString ConnectionStore::connectionsFile() const {
    return QDir(dataDir()).filePath("connections.json");
}

// 从 JSON 文件加载所有连接到内存
bool ConnectionStore::load() {
    m_conns.clear();
    const QString filePath = connectionsFile();
    QFile f(filePath);
    if (!f.exists()) {
        QDir().mkpath(QFileInfo(filePath).dir().absolutePath());
        return true; // nothing to load yet
    }
    if (!f.open(QIODevice::ReadOnly)) {
        return false;
    }
    const auto doc = QJsonDocument::fromJson(f.readAll());
    if (!doc.isObject()) return false;
    const auto arr = doc.object().value("connections").toArray();
    for (const auto& v : arr) {
        if (v.isObject()) m_conns.push_back(fromJson(v.toObject()));
    }
    return true;
}

// 将内存中的连接写回 JSON 文件
bool ConnectionStore::save() const {
    const QString filePath = connectionsFile();
    QDir().mkpath(QFileInfo(filePath).dir().absolutePath());
    QFile f(filePath);
    if (!f.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
        return false;
    }
    QJsonArray arr;
    for (const auto& c : m_conns) arr.push_back(toJson(c));
    QJsonObject root;
    root["connections"] = arr;
    QJsonDocument doc(root);
    f.write(doc.toJson(QJsonDocument::Indented));
    return true;
}

// 新增或按 id 更新连接（若传入 cfg.id 为空则自动生成 UUID）
void ConnectionStore::addOrUpdate(ConnectionConfig cfg) {
    if (cfg.id.isEmpty()) cfg.id = QUuid::createUuid().toString(QUuid::WithoutBraces);
    for (auto& c : m_conns) {
        if (c.id == cfg.id) {
            c = std::move(cfg);
            save();
            emit changed();
            return;
        }
    }
    m_conns.push_back(std::move(cfg));
    save();
    emit changed();
}

// 删除指定 id 的连接
void ConnectionStore::removeById(const QString& id) {
    for (int i = 0; i < m_conns.size(); ++i) {
        if (m_conns[i].id == id) {
            m_conns.remove(i);
            save();
            emit changed();
            return;
        }
    }
}

// 查找指定 id 的连接，若不存在返回空
std::optional<ConnectionConfig> ConnectionStore::byId(const QString& id) const {
    for (const auto& c : m_conns) {
        if (c.id == id) return c;
    }
    return std::nullopt;
}
