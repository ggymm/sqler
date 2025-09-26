#include "Backend.h"
#include "../core/Config.h"
#include "../db/DbUtil.h"

#include <QVariant>
#include <QVariantList>
#include <QVariantMap>

// 构造：初始化连接缓存并监听存储层变化
Backend::Backend(QObject* parent)
    : QObject(parent) {
    rebuildConnectionsCache();
    QObject::connect(&ConnectionStore::instance(), &ConnectionStore::changed, this, [this]{
        rebuildConnectionsCache();
        emit connectionsChanged();
    });
}

// dataDir 读取：直接来自 Config 单例
QString Backend::dataDir() const {
    return Config::instance().get().dataDir;
}

// 设置 dataDir 并保存
void Backend::setDataDir(const QString& dir) {
    if (dir == dataDir()) return;
    Config::instance().setDataDir(dir);
    Config::instance().save();
    emit dataDirChanged();
}

// 根据 ConnectionStore 的数据重建缓存列表供 QML 显示
void Backend::rebuildConnectionsCache() {
    m_connections.clear();
    const auto& list = ConnectionStore::instance().connections();
    m_connections.reserve(list.size());
    for (const auto& c : list) m_connections.push_back(toMap(c));
}

// ConnectionConfig -> QVariantMap（便于 QML 读取/修改）
QVariantMap Backend::toMap(const ConnectionConfig& c) {
    QVariantMap m;
    m["id"] = c.id;
    m["name"] = c.name;
    m["type"] = c.type;
    m["host"] = c.host;
    m["port"] = c.port;
    m["user"] = c.user;
    m["password"] = c.password;
    m["database"] = c.database;
    return m;
}

// QVariantMap -> ConnectionConfig
ConnectionConfig Backend::fromMap(const QVariantMap& m) {
    ConnectionConfig c;
    c.id = m.value("id").toString();
    c.name = m.value("name").toString();
    c.type = m.value("type").toString();
    c.host = m.value("host").toString();
    c.port = m.value("port").toInt();
    c.user = m.value("user").toString();
    c.password = m.value("password").toString();
    c.database = m.value("database").toString();
    return c;
}

// 重新加载磁盘上的连接并刷新缓存/通知前端
void Backend::refreshConnections() {
    ConnectionStore::instance().load();
    rebuildConnectionsCache();
    emit connectionsChanged();
}

// 根据 id 读取单条连接信息
QVariantMap Backend::getConnection(const QString& id) const {
    auto opt = ConnectionStore::instance().byId(id);
    if (!opt) return {};
    return toMap(*opt);
}

// 保存连接（新增或更新），返回 id
QString Backend::saveConnection(const QVariantMap& m) {
    auto c = fromMap(m);
    ConnectionStore::instance().addOrUpdate(c);
    return c.id;
}

// 删除连接（按 id）
bool Backend::deleteConnection(const QString& id) {
    ConnectionStore::instance().removeById(id);
    return true;
}

// 测试连接：调用 DbUtil 并返回 { ok, error }
QVariantMap Backend::testConnection(const QVariantMap& m) const {
    QVariantMap res;
    QString err;
    if (DbUtil::testConnection(fromMap(m), err)) {
        res["ok"] = true;
    } else {
        res["ok"] = false;
        res["error"] = err;
    }
    return res;
}

// 列出表名：调用 DbUtil 并返回 { ok, tables, error }
QVariantMap Backend::listTables(const QVariantMap& m) const {
    QVariantMap res;
    QString err;
    const auto tables = DbUtil::listTables(fromMap(m), err);
    if (!err.isEmpty()) {
        res["ok"] = false;
        res["error"] = err;
    } else {
        res["ok"] = true;
        QVariantList v;
        for (const auto& t : tables) v.push_back(t);
        res["tables"] = v;
    }
    return res;
}

// 执行 SQL 并返回结构化结果
QVariantMap Backend::executeSql(const QVariantMap& m, const QString& sql) const {
    const auto cfg = fromMap(m);
    return DbUtil::executeSql(cfg, sql);
}
