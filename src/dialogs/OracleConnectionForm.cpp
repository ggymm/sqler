#include "OracleConnectionForm.h"

#include "../components/GLabel.h"
#include "../components/GLineEdit.h"
#include "../components/GSpinBox.h"

#include <QFormLayout>

OracleConnectionForm::OracleConnectionForm(QWidget* parent) : ConnectionFormBase(parent)
{
    setupUI();
}

void OracleConnectionForm::setupUI()
{
    // Connection Name
    m_nameEdit = new GLineEdit(this);
    m_nameEdit->setText("Oracle 连接");
    m_nameEdit->setPlaceholderText("我的Oracle连接");
    m_formLayout->addRow(new GLabel("连接名称:"), m_nameEdit);

    // Host Address
    m_hostEdit = new GLineEdit(this);
    m_hostEdit->setText("localhost");
    m_hostEdit->setPlaceholderText("localhost 或 IP 地址");
    m_formLayout->addRow(new GLabel("主机地址:"), m_hostEdit);

    // Port
    m_portSpin = new GSpinBox(this);
    m_portSpin->setRange(1, 65535);
    m_portSpin->setValue(1521);
    m_portSpin->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    m_formLayout->addRow(new GLabel("端口号:"), m_portSpin);

    // Username
    m_usernameEdit = new GLineEdit(this);
    m_usernameEdit->setPlaceholderText("数据库用户名 (例如: system)");
    m_formLayout->addRow(new GLabel("用户名:"), m_usernameEdit);

    // Password
    m_passwordEdit = new GLineEdit(this);
    m_passwordEdit->setEchoMode(QLineEdit::Password);
    m_passwordEdit->setPlaceholderText("数据库密码");
    m_formLayout->addRow(new GLabel("密码:"), m_passwordEdit);

    // Service Name / SID
    m_serviceNameEdit = new GLineEdit(this);
    m_serviceNameEdit->setPlaceholderText("服务名或SID (例如: ORCL)");
    m_formLayout->addRow(new GLabel("服务名/SID:"), m_serviceNameEdit);
}

QVariantMap OracleConnectionForm::getConnectionData() const
{
    QVariantMap data;
    data["type"] = "oracle";
    data["name"] = m_nameEdit->text();
    data["host"] = m_hostEdit->text();
    data["port"] = m_portSpin->value();
    data["username"] = m_usernameEdit->text();
    data["password"] = m_passwordEdit->text();
    data["serviceName"] = m_serviceNameEdit->text();
    return data;
}

bool OracleConnectionForm::validateInput() const
{
    return !m_nameEdit->text().isEmpty() && !m_hostEdit->text().isEmpty() && !m_usernameEdit->text().isEmpty();
}
