#pragma once

#include <QWidget>
#include <QString>
#include <optional>
#include "../core/ConnectionStore.h" // ConnectionConfig 定义

class QTreeWidget;

// 旧版 Widgets 的“表”页（已由 QML 替代）。
// 接受一个连接配置，调用 DbUtil 列出表名并显示在树上。
class TablesPane : public QWidget {
    Q_OBJECT
public:
    explicit TablesPane(QWidget* parent = nullptr);
    void setConnection(const ConnectionConfig& cfg); // 设置当前连接并加载

private:
    void reload(); // 根据 m_cfg 重新加载表数据

private:
    QTreeWidget* m_tree {nullptr};
    std::optional<ConnectionConfig> m_cfg; // 当前连接配置
};
