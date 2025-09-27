#include "MySQLConnectionForm.h"
#include <QFormLayout>
#include <QLineEdit>
#include <QSpinBox>
#include <QLabel>
#include <QSizePolicy>

MySQLConnectionForm::MySQLConnectionForm(QWidget* parent)
    : ConnectionFormBase(parent) {
    setupUI();
}

void MySQLConnectionForm::setupUI() {
    // Connection Name
    m_nameEdit = new QLineEdit(this);
    m_nameEdit->setText("MySQL 连接");
    m_nameEdit->setPlaceholderText("我的MySQL连接");
    m_formLayout->addRow("连接名称:", m_nameEdit);

    // Host Address
    m_hostEdit = new QLineEdit(this);
    m_hostEdit->setText("localhost");
    m_hostEdit->setPlaceholderText("localhost 或 IP 地址");
    m_formLayout->addRow("主机地址:", m_hostEdit);

    // Port
    m_portSpin = new QSpinBox(this);
    m_portSpin->setRange(1, 65535);
    m_portSpin->setValue(3306);
    m_portSpin->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    m_formLayout->addRow("端口号:", m_portSpin);

    // Username
    m_usernameEdit = new QLineEdit(this);
    m_usernameEdit->setPlaceholderText("数据库用户名 (例如: root)");
    m_formLayout->addRow("用户名:", m_usernameEdit);

    // Password
    m_passwordEdit = new QLineEdit(this);
    m_passwordEdit->setEchoMode(QLineEdit::Password);
    m_passwordEdit->setPlaceholderText("数据库密码");
    m_formLayout->addRow("密码:", m_passwordEdit);

    // Database
    m_databaseEdit = new QLineEdit(this);
    m_databaseEdit->setPlaceholderText("数据库名称 (可选)");
    m_formLayout->addRow("数据库名:", m_databaseEdit);
}

QVariantMap MySQLConnectionForm::getConnectionData() const {
    QVariantMap data;
    data["type"] = "mysql";
    data["name"] = m_nameEdit->text();
    data["host"] = m_hostEdit->text();
    data["port"] = m_portSpin->value();
    data["username"] = m_usernameEdit->text();
    data["password"] = m_passwordEdit->text();
    data["database"] = m_databaseEdit->text();
    return data;
}

bool MySQLConnectionForm::validateInput() const {
    return !m_nameEdit->text().isEmpty() &&
           !m_hostEdit->text().isEmpty() &&
           !m_usernameEdit->text().isEmpty();
}