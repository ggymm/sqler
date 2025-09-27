#include "MongoDBConnectionForm.h"
#include <QFormLayout>
#include <QLineEdit>
#include <QSpinBox>
#include <QLabel>
#include <QSizePolicy>

MongoDBConnectionForm::MongoDBConnectionForm(QWidget* parent)
    : ConnectionFormBase(parent) {
    setupUI();
}

void MongoDBConnectionForm::setupUI() {
    // Connection Name
    m_nameEdit = new QLineEdit(this);
    m_nameEdit->setText("MongoDB 数据库");
    m_nameEdit->setPlaceholderText("我的MongoDB连接");
    m_formLayout->addRow("连接名称:", m_nameEdit);

    // Connection String
    m_connectionStringEdit = new QLineEdit(this);
    m_connectionStringEdit->setPlaceholderText("mongodb://localhost:27017/mydb (可选)");
    m_formLayout->addRow("连接字符串:", m_connectionStringEdit);

    // Host Address
    m_hostEdit = new QLineEdit(this);
    m_hostEdit->setText("localhost");
    m_hostEdit->setPlaceholderText("localhost 或 IP 地址");
    m_formLayout->addRow("主机地址:", m_hostEdit);

    // Port
    m_portSpin = new QSpinBox(this);
    m_portSpin->setRange(1, 65535);
    m_portSpin->setValue(27017);
    m_portSpin->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    m_formLayout->addRow("端口号:", m_portSpin);

    // Username
    m_usernameEdit = new QLineEdit(this);
    m_usernameEdit->setPlaceholderText("数据库用户名 (可选)");
    m_formLayout->addRow("用户名:", m_usernameEdit);

    // Password
    m_passwordEdit = new QLineEdit(this);
    m_passwordEdit->setEchoMode(QLineEdit::Password);
    m_passwordEdit->setPlaceholderText("数据库密码 (可选)");
    m_formLayout->addRow("密码:", m_passwordEdit);

    // Database
    m_databaseEdit = new QLineEdit(this);
    m_databaseEdit->setPlaceholderText("数据库名称");
    m_formLayout->addRow("数据库名:", m_databaseEdit);
}

QVariantMap MongoDBConnectionForm::getConnectionData() const {
    QVariantMap data;
    data["type"] = "mongodb";
    data["name"] = m_nameEdit->text();
    data["connectionString"] = m_connectionStringEdit->text();
    data["host"] = m_hostEdit->text();
    data["port"] = m_portSpin->value();
    data["username"] = m_usernameEdit->text();
    data["password"] = m_passwordEdit->text();
    data["database"] = m_databaseEdit->text();
    return data;
}

bool MongoDBConnectionForm::validateInput() const {
    return !m_nameEdit->text().isEmpty() &&
           (!m_connectionStringEdit->text().isEmpty() || !m_hostEdit->text().isEmpty());
}