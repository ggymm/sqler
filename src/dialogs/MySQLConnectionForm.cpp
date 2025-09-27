#include "MySQLConnectionForm.h"
#include <QFormLayout>
#include <QLineEdit>
#include <QSpinBox>
#include <QLabel>

MySQLConnectionForm::MySQLConnectionForm(QWidget* parent)
    : ConnectionFormBase(parent) {
    setupUI();
}

void MySQLConnectionForm::setupUI() {
    m_nameEdit = new QLineEdit(this);
    m_nameEdit->setText("MySQL 连接");
    m_nameEdit->setPlaceholderText("我的MySQL连接");
    m_formLayout->addRow("连接名称:", m_nameEdit);

    m_hostEdit = new QLineEdit(this);
    m_hostEdit->setText("localhost");
    m_hostEdit->setPlaceholderText("localhost");
    m_formLayout->addRow("主机地址:", m_hostEdit);

    m_portSpin = new QSpinBox(this);
    m_portSpin->setRange(1, 65535);
    m_portSpin->setValue(3306);
    m_formLayout->addRow("端口:", m_portSpin);

    m_usernameEdit = new QLineEdit(this);
    m_usernameEdit->setPlaceholderText("root");
    m_formLayout->addRow("用户名:", m_usernameEdit);

    m_passwordEdit = new QLineEdit(this);
    m_passwordEdit->setEchoMode(QLineEdit::Password);
    m_passwordEdit->setPlaceholderText("请输入密码");
    m_formLayout->addRow("密码:", m_passwordEdit);

    m_databaseEdit = new QLineEdit(this);
    m_databaseEdit->setPlaceholderText("请输入数据库名称");
    m_formLayout->addRow("数据库:", m_databaseEdit);
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