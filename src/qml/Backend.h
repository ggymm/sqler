#pragma once

#include <QObject>
#include <QVariant>
#include <QVariantList>
#include <QVariantMap>
#include "../core/ConnectionStore.h"

// QML 后端桥接对象：提供给 QML 调用的接口
// - 暴露连接列表、数据目录配置
// - 提供增删改查、测试连接、列出表等方法
class Backend : public QObject {
    Q_OBJECT
    // 连接列表（QVariantList，每个元素是 QVariantMap）
    Q_PROPERTY(QVariantList connections READ connections NOTIFY connectionsChanged)
    // 数据目录（TOML 配置项 data_dir）
    Q_PROPERTY(QString dataDir READ dataDir WRITE setDataDir NOTIFY dataDirChanged)
public:
    explicit Backend(QObject* parent = nullptr);

    // 读取连接列表缓存（由 rebuildConnectionsCache 构建）
    QVariantList connections() const { return m_connections; }
    // 读取/设置数据目录（会保存到 config.toml）
    QString dataDir() const;
    void setDataDir(const QString& dir);

    // QML 可调用方法
    Q_INVOKABLE void refreshConnections();
    Q_INVOKABLE QVariantMap getConnection(const QString& id) const;
    Q_INVOKABLE QString saveConnection(const QVariantMap& m); // 成功返回 id
    Q_INVOKABLE bool deleteConnection(const QString& id);
    Q_INVOKABLE QVariantMap testConnection(const QVariantMap& m) const; // { ok:bool, error:string }
    Q_INVOKABLE QVariantMap listTables(const QVariantMap& m) const; // { ok:bool, tables:array, error:string }
    Q_INVOKABLE QVariantMap executeSql(const QVariantMap& m, const QString& sql) const; // { ok, error, columns, rows, affected }

signals:
    void connectionsChanged(); // 连接列表变更通知 QML 刷新
    void dataDirChanged();     // 数据目录变更通知

private:
    // 从 ConnectionStore 同步缓存列表到 m_connections
    void rebuildConnectionsCache();
    // 连接配置 <-> QVariantMap 互转（用于与 QML 传参）
    static QVariantMap toMap(const ConnectionConfig& c);
    static ConnectionConfig fromMap(const QVariantMap& m);

private:
    QVariantList m_connections; // 连接列表缓存
};
