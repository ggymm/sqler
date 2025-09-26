// 旧版 Widgets 主窗口实现（已由 QML 前端替代）。
// 此实现保留以便对比/迁移。
#include "MainWindow.h"

#include <QApplication>
#include "../core/ConnectionStore.h"
#include "NewConnectionDialog.h"
#include "../db/DbUtil.h"
#include <QHBoxLayout>
#include <QHeaderView>
#include <QLabel>
#include <QMenu>
#include <QSplitter>
#include <QTreeWidget>
#include <QVBoxLayout>
#include <QWidget>
#include <QTabWidget>
#include <QMessageBox>
#include "TablesPane.h"

MainWindow::MainWindow(QWidget* parent)
    : QMainWindow(parent) {
    setWindowTitle("SQLER");
    resize(1080, 720);

    auto* central = new QWidget(this);
    auto* vbox = new QVBoxLayout(central);
    vbox->setContentsMargins(0, 0, 0, 0);
    vbox->setSpacing(0);

    // 顶部栏（48px），当前不含控件
    QWidget* topBar = createTopBar();
    vbox->addWidget(topBar);

    // 中部：水平分割，左侧连接，右侧标签页
    auto* splitter = new QSplitter(Qt::Horizontal, central);

    QWidget* leftPanel = createConnectionsPanel();
    m_tabs = new QTabWidget(splitter);
    auto* defaultPane = new TablesPane(m_tabs); // 默认表页（无连接）
    m_tabs->addTab(defaultPane, QStringLiteral("表"));

    splitter->addWidget(leftPanel);
    splitter->addWidget(m_tabs);

    // 初始尺寸
    leftPanel->setMinimumWidth(220);
    QList<int> sizes;
    sizes << 280 << 800;
    splitter->setSizes(sizes);

    vbox->addWidget(splitter, /*stretch*/ 1);
    setCentralWidget(central);
}

QWidget* MainWindow::createTopBar() {
    auto* bar = new QWidget(this);
    bar->setObjectName("TopBar");
    bar->setFixedHeight(48);
    bar->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);

    // 预留横向布局，便于未来添加图标按钮
    auto* h = new QHBoxLayout(bar);
    h->setContentsMargins(8, 0, 8, 0);
    h->setSpacing(8);

    // 使用样式表绘制底部细线
    bar->setStyleSheet("#TopBar { border-bottom: 1px solid rgba(0,0,0,0.08); }");
    return bar;
}

QWidget* MainWindow::createConnectionsPanel() {
    auto* panel = new QWidget(this);
    auto* v = new QVBoxLayout(panel);
    v->setContentsMargins(0, 0, 0, 0);
    v->setSpacing(0);

    m_connectionsTree = new QTreeWidget(panel);
    m_connectionsTree->setObjectName("ConnectionsTree");
    m_connectionsTree->setHeaderHidden(true);
    m_connectionsTree->setContextMenuPolicy(Qt::CustomContextMenu);
    connect(m_connectionsTree, &QTreeWidget::customContextMenuRequested,
            this, &MainWindow::onConnectionsContextMenuRequested);
    connect(m_connectionsTree, &QTreeWidget::itemDoubleClicked, this, [this](QTreeWidgetItem* item, int){
        if (!item || item == m_rootItem) return;
        const QString id = item->data(0, Qt::UserRole + 1).toString();
        if (!id.isEmpty()) openConnectionTab(id);
    });

    m_rootItem = new QTreeWidgetItem(QStringList() << QStringLiteral("连接管理"));
    m_rootItem->setFlags(m_rootItem->flags() & ~Qt::ItemIsEditable);
    m_connectionsTree->addTopLevelItem(m_rootItem);
    reloadConnectionsTree();
    connect(&ConnectionStore::instance(), &ConnectionStore::changed,
            this, &MainWindow::onStoreChanged);

    v->addWidget(m_connectionsTree);
    return panel;
}

void MainWindow::onConnectionsContextMenuRequested(const QPoint& pos) {
    if (!m_connectionsTree) return; // 防御
    QMenu menu(this);
    QTreeWidgetItem* item = m_connectionsTree->itemAt(pos);
    if (!item || item == m_rootItem) {
        QAction* newConn = menu.addAction(QStringLiteral("新建连接"));
        QAction* refresh = menu.addAction(QStringLiteral("刷新"));
        QAction* chosen = menu.exec(m_connectionsTree->viewport()->mapToGlobal(pos));
        if (chosen == newConn) {
            NewConnectionDialog dlg(this);
            dlg.exec();
        } else if (chosen == refresh) {
            reloadConnectionsTree();
        }
        return;
    }
    const QString id = item->data(0, Qt::UserRole + 1).toString();
    QAction* open = menu.addAction(QStringLiteral("打开"));
    QAction* test = menu.addAction(QStringLiteral("测试连接"));
    QAction* edit = menu.addAction(QStringLiteral("编辑连接"));
    QAction* del = menu.addAction(QStringLiteral("删除连接"));
    QAction* chosen = menu.exec(m_connectionsTree->viewport()->mapToGlobal(pos));
    if (chosen == open) {
        openConnectionTab(id);
    } else if (chosen == test) {
        auto cfgOpt = ConnectionStore::instance().byId(id);
        if (!cfgOpt) return;
        QString err;
        if (DbUtil::testConnection(*cfgOpt, err)) {
            QMessageBox::information(this, QStringLiteral("测试连接"), QStringLiteral("连接成功"));
        } else {
            QMessageBox::warning(this, QStringLiteral("测试连接失败"), err);
        }
    } else if (chosen == edit) {
        auto cfgOpt = ConnectionStore::instance().byId(id);
        if (!cfgOpt) return;
        NewConnectionDialog dlg(this);
        dlg.setInitialConfig(*cfgOpt);
        dlg.exec();
    } else if (chosen == del) {
        auto cfgOpt = ConnectionStore::instance().byId(id);
        if (!cfgOpt) return;
        const auto name = cfgOpt->name.isEmpty() ? cfgOpt->id : cfgOpt->name;
        if (QMessageBox::question(this, QStringLiteral("删除连接"),
                                  QStringLiteral("确定删除连接 '%1' 吗？").arg(name)) == QMessageBox::Yes) {
            ConnectionStore::instance().removeById(id);
            closeConnectionTab(id);
        }
    }
}

void MainWindow::reloadConnectionsTree() {
    if (!m_connectionsTree || !m_rootItem) return;
    m_rootItem->takeChildren();
    const auto& list = ConnectionStore::instance().connections();
    for (const auto& c : list) {
        auto* item = new QTreeWidgetItem(m_rootItem, QStringList() << (c.name.isEmpty() ? c.id : c.name));
        item->setData(0, Qt::UserRole + 1, c.id);
        item->setToolTip(0, QString("%1@%2:%3 [%4]")
                                  .arg(c.user)
                                  .arg(c.host)
                                  .arg(c.port)
                                  .arg(c.type));
    }
    m_connectionsTree->expandItem(m_rootItem);
}

void MainWindow::onStoreChanged() {
    reloadConnectionsTree();
}

void MainWindow::openConnectionTab(const QString& id) {
    if (m_connTabs.contains(id)) {
        m_tabs->setCurrentIndex(m_connTabs.value(id));
        return;
    }
    auto cfgOpt = ConnectionStore::instance().byId(id);
    if (!cfgOpt) return;
    auto* pane = new TablesPane(m_tabs);
    pane->setConnection(*cfgOpt);
    const QString title = QStringLiteral("表 - %1").arg(cfgOpt->name.isEmpty() ? cfgOpt->id : cfgOpt->name);
    int idx = m_tabs->addTab(pane, title);
    m_connTabs.insert(id, idx);
    m_tabs->setCurrentIndex(idx);
}

void MainWindow::closeConnectionTab(const QString& id) {
    if (!m_connTabs.contains(id)) return;
    int idx = m_connTabs.take(id);
    QWidget* w = m_tabs->widget(idx);
    m_tabs->removeTab(idx);
    if (w) w->deleteLater();
    // Shift indices > removed index
    for (auto it = m_connTabs.begin(); it != m_connTabs.end(); ++it) {
        if (it.value() > idx) it.value() = it.value() - 1;
    }
}
