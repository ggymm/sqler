#include "RedisConnectionForm.h"
#include <QFormLayout>
#include <QLineEdit>
#include <QSpinBox>
#include <QLabel>
#include <QSizePolicy>

RedisConnectionForm::RedisConnectionForm(QWidget* parent)
    : ConnectionFormBase(parent) {
    setupUI();
}

void RedisConnectionForm::setupUI() {
    // Connection Name
    m_nameEdit = new QLineEdit(this);
    m_nameEdit->setText("Redis 缓存");
    m_nameEdit->setPlaceholderText("我的Redis连接");
    m_formLayout->addRow("连接名称:", m_nameEdit);

    // Host Address
    m_hostEdit = new QLineEdit(this);
    m_hostEdit->setText("localhost");
    m_hostEdit->setPlaceholderText("localhost 或 IP 地址");
    m_formLayout->addRow("主机地址:", m_hostEdit);

    // Port
    m_portSpin = new QSpinBox(this);
    m_portSpin->setRange(1, 65535);
    m_portSpin->setValue(6379);
    m_portSpin->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    m_formLayout->addRow("端口号:", m_portSpin);

    // Username
    m_usernameEdit = new QLineEdit(this);
    m_usernameEdit->setPlaceholderText("用户名 (可选, Redis 6.0+)");
    m_formLayout->addRow("用户名:", m_usernameEdit);

    // Password
    m_passwordEdit = new QLineEdit(this);
    m_passwordEdit->setEchoMode(QLineEdit::Password);
    m_passwordEdit->setPlaceholderText("认证密码 (可选)");
    m_formLayout->addRow("密码:", m_passwordEdit);

    // Database Index
    m_databaseSpin = new QSpinBox(this);
    m_databaseSpin->setRange(0, 15);
    m_databaseSpin->setValue(0);
    m_databaseSpin->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    m_formLayout->addRow("数据库索引:", m_databaseSpin);
}

QVariantMap RedisConnectionForm::getConnectionData() const {
    QVariantMap data;
    data["type"] = "redis";
    data["name"] = m_nameEdit->text();
    data["host"] = m_hostEdit->text();
    data["port"] = m_portSpin->value();
    data["username"] = m_usernameEdit->text();
    data["password"] = m_passwordEdit->text();
    data["database"] = m_databaseSpin->value();
    return data;
}

bool RedisConnectionForm::validateInput() const {
    return !m_nameEdit->text().isEmpty() &&
           !m_hostEdit->text().isEmpty();
}