// 旧版 Widgets “新建连接”对话框实现（现已由 QML 替代）。
// 功能：选择 MySQL -> 填写连接信息 -> 测试/保存。
#include "NewConnectionDialog.h"
#include "../core/ConnectionStore.h"
#include "../db/DbUtil.h"

#include <QDialogButtonBox>
#include <QPushButton>
#include <QAbstractButton>
#include <QFormLayout>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QMessageBox>
#include <QSpinBox>
#include <QStackedWidget>
#include <QVBoxLayout>

NewConnectionDialog::NewConnectionDialog(QWidget* parent)
    : QDialog(parent) {
    setWindowTitle(QStringLiteral("新建连接")); // 对话框标题
    resize(560, 420);

    auto* v = new QVBoxLayout(this);
    m_stack = new QStackedWidget(this); // 两步页面容器
    v->addWidget(m_stack, 1);

    buildTypePage();
    buildMySqlPage();
    m_stack->setCurrentIndex(0);

    m_buttons = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel, this);
    auto* testBtn = m_buttons->addButton(QStringLiteral("测试连接"), QDialogButtonBox::ActionRole);
    connect(m_buttons, &QDialogButtonBox::accepted, this, &NewConnectionDialog::onAccept);
    connect(m_buttons, &QDialogButtonBox::rejected, this, &NewConnectionDialog::reject);
    connect(testBtn, &QAbstractButton::clicked, this, &NewConnectionDialog::onTest);
    v->addWidget(m_buttons);
}

void NewConnectionDialog::buildTypePage() {
    auto* page = new QWidget(this);
    auto* v = new QVBoxLayout(page);
    auto* tip = new QLabel(QStringLiteral("选择数据库类型"), page); // 顶部提示
    v->addWidget(tip);

    m_typeList = new QListWidget(page); // 仅提供 MySQL 选项
    m_typeList->addItem(QStringLiteral("MySQL"));
    m_typeList->setSelectionMode(QAbstractItemView::SingleSelection);
    v->addWidget(m_typeList, 1);

    auto toDetails = [this](QListWidgetItem* item) {
        if (!item) return;
        if (item->text() == QStringLiteral("MySQL")) {
            m_selectedType = QStringLiteral("mysql");
            m_stack->setCurrentIndex(1);
        }
    };
    connect(m_typeList, &QListWidget::itemActivated, this, toDetails);
    connect(m_typeList, &QListWidget::itemClicked, this, toDetails);

    m_stack->addWidget(page);
}

void NewConnectionDialog::buildMySqlPage() {
    auto* page = new QWidget(this);
    auto* form = new QFormLayout(page);
    form->setLabelAlignment(Qt::AlignRight);

    m_name = new QLineEdit(page);
    m_host = new QLineEdit(page);
    m_port = new QSpinBox(page);
    m_user = new QLineEdit(page);
    m_password = new QLineEdit(page);
    m_database = new QLineEdit(page);

    m_port->setRange(1, 65535);   // 端口范围
    m_port->setValue(3306);
    m_password->setEchoMode(QLineEdit::Password); // 密码隐藏

    m_name->setPlaceholderText(QStringLiteral("例如：本地 MySQL"));
    m_host->setPlaceholderText(QStringLiteral("localhost"));
    m_user->setPlaceholderText(QStringLiteral("root"));

    form->addRow(QStringLiteral("名称"), m_name);
    form->addRow(QStringLiteral("主机"), m_host);
    form->addRow(QStringLiteral("端口"), m_port);
    form->addRow(QStringLiteral("用户名"), m_user);
    form->addRow(QStringLiteral("密码"), m_password);
    form->addRow(QStringLiteral("数据库(可选)"), m_database);

    m_stack->addWidget(page);
}

bool NewConnectionDialog::resultConfig(ConnectionConfig& out) const {
    if (m_selectedType != QLatin1String("mysql")) return false;
    if (!m_name || !m_host || !m_port || !m_user) return false;
    out.type = m_selectedType;
    out.name = m_name->text().trimmed();
    out.host = m_host->text().trimmed();
    out.port = m_port->value();
    out.user = m_user->text();
    out.password = m_password ? m_password->text() : QString();
    out.database = m_database ? m_database->text() : QString();
    // 基础校验：名称与主机不能为空
    if (out.name.isEmpty()) return false;
    if (out.host.isEmpty()) return false;
    return true;
}

void NewConnectionDialog::onTypeActivated() {
    // Not used; kept for potential future wiring
}

void NewConnectionDialog::onAccept() {
    if (m_stack->currentIndex() == 0) {
        // if still on type page, try to advance when Ok is pressed
        auto* item = m_typeList && m_typeList->currentItem() ? m_typeList->currentItem() : nullptr;
        if (item && item->text() == QStringLiteral("MySQL")) {
            m_selectedType = QStringLiteral("mysql");
            m_stack->setCurrentIndex(1);
            return;
        }
    }
    ConnectionConfig cfg;
    if (!resultConfig(cfg)) {
        // 简单校验未通过，不关闭对话框
        return;
    }
    if (!m_editingId.isEmpty()) cfg.id = m_editingId;
    ConnectionStore::instance().addOrUpdate(std::move(cfg));
    accept();
}

void NewConnectionDialog::onTest() {
    ConnectionConfig cfg;
    if (!resultConfig(cfg)) return;
    QString err;
    if (DbUtil::testConnection(cfg, err)) {
        QMessageBox::information(this, QStringLiteral("测试连接"), QStringLiteral("连接成功"));
    } else {
        QMessageBox::warning(this, QStringLiteral("测试连接失败"), err);
    }
}

void NewConnectionDialog::setInitialConfig(const ConnectionConfig& cfg) {
    m_editingId = cfg.id;
    m_selectedType = cfg.type; // 仅 MySQL 页面
    // For now only mysql page exists
    if (m_stack) m_stack->setCurrentIndex(1);
    if (m_name) m_name->setText(cfg.name);
    if (m_host) m_host->setText(cfg.host);
    if (m_port && cfg.port > 0) m_port->setValue(cfg.port);
    if (m_user) m_user->setText(cfg.user);
    if (m_password) m_password->setText(cfg.password);
    if (m_database) m_database->setText(cfg.database);
}
