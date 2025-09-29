#include "RedisConnectionForm.h"

#include "../components/GLabel.h"
#include "../components/GLineEdit.h"
#include "../components/GSpinBox.h"

#include <QFormLayout>

RedisConnectionForm::RedisConnectionForm(QWidget* parent) : ConnectionFormBase(parent) { RedisConnectionForm::setupUI(); }

void RedisConnectionForm::setupUI() {
    // Connection Name
    m_nameEdit = new GLineEdit(this);
    m_nameEdit->setText("Redis 缓存");
    m_nameEdit->setPlaceholderText("我的Redis连接");
    m_formLayout->addRow(new GLabel("连接名称:"), m_nameEdit);

    // Host Address
    m_hostEdit = new GLineEdit(this);
    m_hostEdit->setText("localhost");
    m_hostEdit->setPlaceholderText("localhost 或 IP 地址");
    m_formLayout->addRow(new GLabel("主机地址:"), m_hostEdit);

    // Port
    m_portSpin = new GSpinBox(this);
    m_portSpin->setRange(1, 65535);
    m_portSpin->setValue(6379);
    m_portSpin->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    m_formLayout->addRow(new GLabel("端口号:"), m_portSpin);

    // Username
    m_usernameEdit = new GLineEdit(this);
    m_usernameEdit->setPlaceholderText("用户名 (可选, Redis 6.0+)");
    m_formLayout->addRow(new GLabel("用户名:"), m_usernameEdit);

    // Password
    m_passwordEdit = new GLineEdit(this);
    m_passwordEdit->setEchoMode(QLineEdit::Password);
    m_passwordEdit->setPlaceholderText("认证密码 (可选)");
    m_formLayout->addRow(new GLabel("密码:"), m_passwordEdit);

    // Database Index
    m_databaseSpin = new GSpinBox(this);
    m_databaseSpin->setRange(0, 15);
    m_databaseSpin->setValue(0);
    m_databaseSpin->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    m_formLayout->addRow(new GLabel("数据库索引:"), m_databaseSpin);
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

bool RedisConnectionForm::validateInput() const { return !m_nameEdit->text().isEmpty() && !m_hostEdit->text().isEmpty(); }
