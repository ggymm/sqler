#pragma once

#include <QString>
#include <QStringList>
#include <QVariant>
#include <QVector>
#include <optional>
#include <memory>

struct ConnectionConfig;

// 通用执行结果：用于 SQL 类数据库执行查询
struct QueryResult {
    bool ok {false};
    QString error;            // 错误信息（ok=false时）
    QStringList columns;      // 列名（仅 SELECT 有效）
    QVector<QVariantList> rows; // 行数据（仅 SELECT 有效）
    int affected {0};         // 受影响行数（非 SELECT）
};

// 数据库驱动抽象接口：不同类型数据库通过该接口实现
class IDbDriver {
public:
    virtual ~IDbDriver() = default;

    // 测试连接是否可用
    virtual bool testConnection(const ConnectionConfig& cfg, QString& error) = 0;

    // 列出表名（非 SQL 型数据库可返回不支持）
    virtual QStringList listTables(const ConnectionConfig& cfg, QString& error) { Q_UNUSED(cfg); error = QStringLiteral("当前数据库不支持列出表"); return {}; }

    // 执行 SQL（非 SQL 型数据库可返回不支持）
    virtual QueryResult executeSql(const ConnectionConfig& cfg, const QString& sql) { Q_UNUSED(cfg); Q_UNUSED(sql); return QueryResult{false, QStringLiteral("当前数据库不支持执行 SQL"), {}, {}, 0}; }
};

// 工厂方法：根据 ConnectionConfig::type 返回合适的驱动
std::unique_ptr<IDbDriver> createDriverFor(const QString& type);
