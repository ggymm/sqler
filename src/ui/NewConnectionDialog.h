// 旧版 Widgets “新建连接”对话框（已由 QML 版本替换）
// 两步：选择类型 -> 填写 MySQL 连接信息（当前仅支持 MySQL）
#pragma once

#include <QDialog>

struct ConnectionConfig; // 前置声明：连接配置结构体

class QStackedWidget;
class QListWidget;
class QLineEdit;
class QSpinBox;
class QDialogButtonBox;

class NewConnectionDialog : public QDialog {
    Q_OBJECT
public:
    explicit NewConnectionDialog(QWidget* parent = nullptr);

    // 输出表单内容为 ConnectionConfig；返回 false 表示校验未通过
    bool resultConfig(ConnectionConfig& out) const;
    // 以已有配置初始化（用于编辑场景）
    void setInitialConfig(const ConnectionConfig& cfg);

private slots:
    void onTypeActivated(); // 类型页激活（保留预留）
    void onAccept();        // 点击确定
    void onTest();          // 测试连接

private:
    void buildTypePage();   // 构建类型选择页
    void buildMySqlPage();  // 构建 MySQL 详情页

private:
    QStackedWidget* m_stack {nullptr};
    QListWidget* m_typeList {nullptr};
    QDialogButtonBox* m_buttons {nullptr};
    // MySQL 表单控件
    QLineEdit* m_name {nullptr};
    QLineEdit* m_host {nullptr};
    QSpinBox* m_port {nullptr};
    QLineEdit* m_user {nullptr};
    QLineEdit* m_password {nullptr};
    QLineEdit* m_database {nullptr};
    QString m_selectedType; // 已选类型（如 "mysql"）
    QString m_editingId;    // 编辑模式下保留原 id
};
