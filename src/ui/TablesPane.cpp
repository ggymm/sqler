#include "TablesPane.h"
#include "../db/DbUtil.h"
#include "../core/ConnectionStore.h"

#include <QHeaderView>
#include <QLabel>
#include <QTreeWidget>
#include <QVBoxLayout>
#include <QMessageBox>

// 构造 UI：仅一个 QTreeWidget 展示表名
TablesPane::TablesPane(QWidget* parent)
    : QWidget(parent) {
    auto* v = new QVBoxLayout(this);
    v->setContentsMargins(0, 0, 0, 0);
    v->setSpacing(0);
    m_tree = new QTreeWidget(this);
    m_tree->setHeaderHidden(true);
    v->addWidget(m_tree);
}

// 设定连接并触发加载
void TablesPane::setConnection(const ConnectionConfig& cfg) {
    m_cfg = cfg;
    reload();
}

// 根据当前连接从数据库读取表名并显示
void TablesPane::reload() {
    m_tree->clear();
    if (!m_cfg.has_value()) {
        m_tree->addTopLevelItem(new QTreeWidgetItem(QStringList() << QStringLiteral("未选择连接")));
        return;
    }
    QString err;
    const auto tables = DbUtil::listTables(*m_cfg, err);
    auto* root = new QTreeWidgetItem(QStringList() << QStringLiteral("表"));
    if (!err.isEmpty()) {
        auto* errItem = new QTreeWidgetItem(QStringList() << QStringLiteral("错误: ") + err);
        m_tree->addTopLevelItem(errItem);
        return;
    }
    for (const auto& t : tables) root->addChild(new QTreeWidgetItem(QStringList() << t));
    m_tree->addTopLevelItem(root);
    m_tree->expandItem(root);
}
