#include "SQLServerConnectionForm.h"
#include "../components/GLabel.h"
#include "../components/GLineEdit.h"
#include "../components/GSpinBox.h"
#include <QFormLayout>
#include <QSizePolicy>

SQLServerConnectionForm::SQLServerConnectionForm(QWidget* parent) : ConnectionFormBase(parent) { setupUI(); }

void SQLServerConnectionForm::setupUI()
{
    // Connection Name
    m_nameEdit = new GLineEdit(this);
    m_nameEdit->setText("SQL Server 连接");
    m_nameEdit->setPlaceholderText("我的SQL Server连接");
    m_formLayout->addRow(new GLabel("连接名称:"), m_nameEdit);

    // Host Address
    m_hostEdit = new GLineEdit(this);
    m_hostEdit->setText("localhost");
    m_hostEdit->setPlaceholderText("localhost 或 IP 地址");
    m_formLayout->addRow(new GLabel("主机地址:"), m_hostEdit);

    // Port
    m_portSpin = new GSpinBox(this);
    m_portSpin->setRange(1, 65535);
    m_portSpin->setValue(1433);
    m_portSpin->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    m_formLayout->addRow(new GLabel("端口号:"), m_portSpin);

    // Instance Name (optional)
    m_instanceEdit = new GLineEdit(this);
    m_instanceEdit->setPlaceholderText("实例名 (可选)");
    m_formLayout->addRow(new GLabel("实例名:"), m_instanceEdit);

    // Username
    m_usernameEdit = new GLineEdit(this);
    m_usernameEdit->setPlaceholderText("数据库用户名 (例如: sa)");
    m_formLayout->addRow(new GLabel("用户名:"), m_usernameEdit);

    // Password
    m_passwordEdit = new GLineEdit(this);
    m_passwordEdit->setEchoMode(QLineEdit::Password);
    m_passwordEdit->setPlaceholderText("数据库密码");
    m_formLayout->addRow(new GLabel("密码:"), m_passwordEdit);

    // Database Name (optional)
    m_databaseEdit = new GLineEdit(this);
    m_databaseEdit->setPlaceholderText("数据库名称 (可选)");
    m_formLayout->addRow(new GLabel("数据库名:"), m_databaseEdit);
}

QVariantMap SQLServerConnectionForm::getConnectionData() const
{
    QVariantMap data;
    data["type"] = "sqlserver";
    data["name"] = m_nameEdit->text();
    data["host"] = m_hostEdit->text();
    data["port"] = m_portSpin->value();
    data["instance"] = m_instanceEdit->text();
    data["username"] = m_usernameEdit->text();
    data["password"] = m_passwordEdit->text();
    data["database"] = m_databaseEdit->text();
    return data;
}

bool SQLServerConnectionForm::validateInput() const
{
    return !m_nameEdit->text().isEmpty() && !m_hostEdit->text().isEmpty() && !m_usernameEdit->text().isEmpty();
}
