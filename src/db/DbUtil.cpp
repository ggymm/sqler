#include "DbUtil.h"
#include "../core/ConnectionStore.h"
#include "Drivers.h"

#include <QVariantMap>

// 将抽象驱动封装为工具函数供上层调用

// 连接测试
bool DbUtil::testConnection(const ConnectionConfig& cfg, QString& error) {
    auto drv = createDriverFor(cfg.type);
    if (!drv) { error = QStringLiteral("暂不支持的数据库类型: %1").arg(cfg.type); return false; }
    return drv->testConnection(cfg, error);
}

// 列表表
QStringList DbUtil::listTables(const ConnectionConfig& cfg, QString& error) {
    auto drv = createDriverFor(cfg.type);
    if (!drv) { error = QStringLiteral("暂不支持的数据库类型: %1").arg(cfg.type); return {}; }
    return drv->listTables(cfg, error);
}

// 执行 SQL（转换为 QVariantMap 便于 QML 使用）
QVariantMap DbUtil::executeSql(const ConnectionConfig& cfg, const QString& sql) {
    QVariantMap out;
    auto drv = createDriverFor(cfg.type);
    if (!drv) { out["ok"] = false; out["error"] = QStringLiteral("暂不支持的数据库类型: %1").arg(cfg.type); return out; }
    auto res = drv->executeSql(cfg, sql);
    out["ok"] = res.ok;
    if (!res.ok) out["error"] = res.error;
    // columns
    QVariantList cols;
    for (const auto& c : res.columns) cols.push_back(c);
    out["columns"] = cols;
    // rows
    QVariantList rows;
    rows.reserve(res.rows.size());
    for (const auto& r : res.rows) rows.push_back(r);
    out["rows"] = rows;
    out["affected"] = res.affected;
    return out;
}
