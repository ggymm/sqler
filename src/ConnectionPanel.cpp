#include "ConnectionPanel.h"
#include "Theme.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QListWidget>
#include <QListWidgetItem>
#include <QPushButton>
#include <QLabel>
#include <QIcon>

ConnectionPanel::ConnectionPanel(QWidget* parent)
    : QWidget(parent) {
    setupUI();
    populateConnections();
    applyTheme();

    connect(&Theme::instance(), &Theme::themeChanged, this, &ConnectionPanel::onThemeChanged);
}

void ConnectionPanel::setupUI() {
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(Theme::Spacing::md, Theme::Spacing::md, Theme::Spacing::md, Theme::Spacing::md);
    layout->setSpacing(Theme::Spacing::md);

    auto* headerLayout = new QHBoxLayout();

    auto* titleLabel = new QLabel("连接管理", this);
    titleLabel->setObjectName("titleLabel");
    headerLayout->addWidget(titleLabel);

    headerLayout->addStretch();

    m_addButton = new QPushButton("+", this);
    m_addButton->setFixedSize(32, 32);
    m_addButton->setObjectName("addButton");
    headerLayout->addWidget(m_addButton);

    layout->addLayout(headerLayout);

    m_connectionsList = new QListWidget(this);
    m_connectionsList->setObjectName("connectionsList");
    layout->addWidget(m_connectionsList);

    connect(m_connectionsList, &QListWidget::itemClicked, this, &ConnectionPanel::onConnectionItemClicked);
}

void ConnectionPanel::populateConnections() {
    createConnectionItem("本地 MySQL", "mysql", true);
    createConnectionItem("测试 PostgreSQL", "postgresql", false);
    createConnectionItem("Redis 缓存", "redis", true);
}

QListWidgetItem* ConnectionPanel::createConnectionItem(const QString& name, const QString& type, bool connected) {
    auto* item = new QListWidgetItem(m_connectionsList);

    auto* widget = new QWidget();
    auto* layout = new QHBoxLayout(widget);
    layout->setContentsMargins(Theme::Spacing::sm, Theme::Spacing::sm, Theme::Spacing::sm, Theme::Spacing::sm);
    layout->setSpacing(Theme::Spacing::sm);

    auto* iconLabel = new QLabel(widget);
    iconLabel->setPixmap(QIcon(QStringLiteral(":/assets/icons/db/%1.svg").arg(type))
                        .pixmap(Theme::Sizes::iconSize, Theme::Sizes::iconSize));
    layout->addWidget(iconLabel);

    auto* textLayout = new QVBoxLayout();
    textLayout->setSpacing(2);

    auto* nameLabel = new QLabel(name, widget);
    nameLabel->setObjectName("connectionName");
    textLayout->addWidget(nameLabel);

    auto* typeLabel = new QLabel(type.toUpper(), widget);
    typeLabel->setObjectName("connectionType");
    textLayout->addWidget(typeLabel);

    layout->addLayout(textLayout);
    layout->addStretch();

    auto* statusLabel = new QLabel("●", widget);
    statusLabel->setObjectName(connected ? "statusConnected" : "statusDisconnected");
    layout->addWidget(statusLabel);

    widget->setLayout(layout);
    item->setSizeHint(QSize(0, 48));
    m_connectionsList->setItemWidget(item, widget);

    return item;
}

void ConnectionPanel::applyTheme() {
    const auto& colors = Theme::instance().colors();

    QString styleSheet = QString(
        "ConnectionPanel {"
        "    background-color: %1;"
        "    border-right: 1px solid %2;"
        "}"
        "QLabel#titleLabel {"
        "    color: %3;"
        "    font-size: 16px;"
        "    font-weight: bold;"
        "}"
        "QPushButton#addButton {"
        "    color: %4;"
        "    background-color: %5;"
        "    border: none;"
        "    border-radius: %6px;"
        "    font-size: 18px;"
        "    font-weight: bold;"
        "}"
        "QPushButton#addButton:hover {"
        "    background-color: %7;"
        "    color: white;"
        "}"
        "QListWidget#connectionsList {"
        "    background-color: transparent;"
        "    border: none;"
        "    outline: none;"
        "}"
        "QLabel#connectionName {"
        "    color: %3;"
        "    font-size: 14px;"
        "    font-weight: 500;"
        "}"
        "QLabel#connectionType {"
        "    color: %8;"
        "    font-size: 12px;"
        "}"
        "QLabel#statusConnected {"
        "    color: %9;"
        "    font-size: 12px;"
        "}"
        "QLabel#statusDisconnected {"
        "    color: %10;"
        "    font-size: 12px;"
        "}"
    ).arg(colors.surface.name()), colors.surface.name(), colors.border.name()), colors.border.name())
     .arg(colors.border.name())
     .arg(Theme::Sizes::borderRadius)
     .arg(colors.primary.name())
     .arg(colors.textSecondary.name())
     .arg(colors.success.name())
     .arg(colors.textMuted.name());

    setStyleSheet(styleSheet);
}

void ConnectionPanel::onThemeChanged() {
    applyTheme();
}

void ConnectionPanel::onConnectionItemClicked(QListWidgetItem* item) {
    if (!item) return;

    // Get the connection widget
    QWidget* connectionWidget = m_connectionsList->itemWidget(item);
    if (!connectionWidget) return;

    // Highlight the selected item
    for (int i = 0; i < m_connectionsList->count(); ++i) {
        QListWidgetItem* listItem = m_connectionsList->item(i);
        QWidget* widget = m_connectionsList->itemWidget(listItem);
        if (widget) {
            widget->setStyleSheet(widget == connectionWidget ?
                "QWidget { background-color: " + Theme::instance().colors().primary.name() + "; }" :
                "");
        }
    }

    // Emit signal that a connection was selected
    emit connectionSelected(item->text());
}