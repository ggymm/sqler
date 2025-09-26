#pragma once

#include <QString>
#include <QStringList>
#include <QVariantMap>

// 前置声明：连接配置结构体
struct ConnectionConfig;

// 数据库工具函数命名空间
// 目前仅支持 MySQL（Qt SQL 驱动名：QMYSQL）
namespace DbUtil {

// 测试连接是否成功
// 参数：cfg 连接配置；error 返回错误信息
// 返回：true 表示连接成功，false 表示失败（并在 error 中返回原因）
bool testConnection(const ConnectionConfig& cfg, QString& error);

// 列出指定连接（当前数据库）的所有表名
// 参数：cfg 连接配置；error 返回错误信息
// 返回：表名列表；若失败返回空列表并在 error 中说明
QStringList listTables(const ConnectionConfig& cfg, QString& error);

// 执行 SQL（仅对 SQL 类数据库有效，如 MySQL）
// 返回：
// - ok/error：执行是否成功与错误信息
// - columns/rows/affected 写入到 QVariantMap 中以便 QML 使用
// Map 结构：{ ok:bool, error:string, columns: string[], rows: any[][], affected: int }
QVariantMap executeSql(const ConnectionConfig& cfg, const QString& sql);

}
