// 旧版 Widgets 主窗口（已由 QML 重构替代）
// 保留代码以便后续迁移/参考。在 Widgets 模式下：
// - 顶部 48px 空白栏
// - 左侧：连接管理树（右键菜单）
// - 右侧：Tab 区域（表列表）
#pragma once

#include <QMainWindow>
#include <QHash>

class QTreeWidget;
class QWidget;
class QSplitter;
class QTreeWidgetItem;
class QTabWidget;

class MainWindow : public QMainWindow {
    Q_OBJECT
public:
    explicit MainWindow(QWidget* parent = nullptr);
    ~MainWindow() override = default;

private slots:
    // 连接树右键菜单响应
    void onConnectionsContextMenuRequested(const QPoint& pos);
    // 存储层发生变化时刷新树
    void onStoreChanged();

private:
    QWidget* createTopBar();                 // 创建顶部栏
    QWidget* createConnectionsPanel();       // 创建左侧连接管理面板
    void reloadConnectionsTree();            // 重新加载连接树数据
    void openConnectionTab(const QString& id);  // 打开指定连接的标签页
    void closeConnectionTab(const QString& id); // 关闭指定连接的标签页

private:
    QTreeWidget* m_connectionsTree {nullptr}; // 左侧树控件
    QTreeWidgetItem* m_rootItem {nullptr};    // 根节点“连接管理”
    QTabWidget* m_tabs {nullptr};             // 右侧 Tab 控件
    QHash<QString, int> m_connTabs;           // connId -> tab index 映射
};
