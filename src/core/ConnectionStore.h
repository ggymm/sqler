#pragma once

#include <QObject>
#include <QString>
#include <QVector>
#include <optional>

// 数据库连接配置
// - id: 唯一标识（UUID）
// - name: 显示名称
// - type: 数据库类型（例如 "mysql"）
// - host/port/user/password/database: 连接必需信息
struct ConnectionConfig {
    QString id;        // 唯一 ID（uuid）
    QString name;      // 连接显示名称
    QString type;      // 类型，如 "mysql"
    QString host;      // 主机名/IP
    int port {0};      // 端口
    QString user;      // 用户名
    QString password;  // 密码（当前明文保存，后续可加密）
    QString database;  // 默认数据库/Schema（可选）
};

// 连接配置存储管理（单例）
// - 使用 JSON 文件（connections.json）持久化所有连接
// - 提供增删改查接口，并通过 changed() 信号通知 UI 刷新
class ConnectionStore : public QObject {
    Q_OBJECT
public:
    // 获取全局实例
    static ConnectionStore& instance();

    // 加载/保存所有连接
    bool load();
    bool save() const;

    // CRUD 接口
    const QVector<ConnectionConfig>& connections() const { return m_conns; }
    void addOrUpdate(ConnectionConfig cfg);       // 新增或更新（按 id 判断）
    void removeById(const QString& id);           // 按 id 删除
    std::optional<ConnectionConfig> byId(const QString& id) const; // 按 id 查询

    // 路径辅助：数据目录/文件
    QString dataDir() const;
    QString connectionsFile() const;

signals:
    // 当连接集合发生变化时发出（新增/更新/删除/加载）
    void changed();

private:
    explicit ConnectionStore(QObject* parent = nullptr);
    QVector<ConnectionConfig> m_conns; // 内存中的连接列表
};
