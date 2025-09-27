#include "MainContent.h"
#include "Theme.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QGridLayout>
#include <QLabel>
#include <QPushButton>
#include <QSpacerItem>
#include <QIcon>

MainContent::MainContent(QWidget* parent)
    : QWidget(parent) {
    setupUI();
    applyTheme();

    connect(&Theme::instance(), &Theme::themeChanged, this, &MainContent::onThemeChanged);
}

void MainContent::setupUI() {
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::lg, Theme::Spacing::lg);
    layout->setSpacing(Theme::Spacing::lg);

    layout->addStretch();

    m_titleLabel = new QLabel("欢迎使用 SQL 数据库管理器", this);
    m_titleLabel->setObjectName("titleLabel");
    m_titleLabel->setAlignment(Qt::AlignCenter);
    layout->addWidget(m_titleLabel);

    m_subtitleLabel = new QLabel("请从左侧选择数据库连接开始使用", this);
    m_subtitleLabel->setObjectName("subtitleLabel");
    m_subtitleLabel->setAlignment(Qt::AlignCenter);
    layout->addWidget(m_subtitleLabel);

    layout->addSpacerItem(new QSpacerItem(0, Theme::Spacing::xl, QSizePolicy::Minimum, QSizePolicy::Fixed));

    auto* actionsLayout = new QHBoxLayout();
    actionsLayout->setSpacing(Theme::Spacing::lg);

    m_newConnectionAction = createQuickActionButton("新建连接", "连接到新的数据库", ":/assets/icons/new-conn.svg");
    actionsLayout->addWidget(m_newConnectionAction);

    m_newQueryAction = createQuickActionButton("新建查询", "创建SQL查询", ":/assets/icons/new-query.svg");
    actionsLayout->addWidget(m_newQueryAction);

    m_browseTablesAction = createQuickActionButton("浏览表", "查看数据库表", ":/assets/icons/table.svg");
    actionsLayout->addWidget(m_browseTablesAction);

    auto* actionsWidget = new QWidget(this);
    actionsWidget->setLayout(actionsLayout);
    layout->addWidget(actionsWidget, 0, Qt::AlignCenter);

    layout->addStretch();
}

QPushButton* MainContent::createQuickActionButton(const QString& title, const QString& description, const QString& iconPath) {
    auto* button = new QPushButton(this);
    button->setFixedSize(200, 120);
    button->setObjectName("quickActionButton");

    auto* layout = new QVBoxLayout(button);
    layout->setAlignment(Qt::AlignCenter);
    layout->setSpacing(Theme::Spacing::sm);

    auto* iconLabel = new QLabel(button);
    iconLabel->setPixmap(QIcon(iconPath).pixmap(32, 32));
    iconLabel->setAlignment(Qt::AlignCenter);
    layout->addWidget(iconLabel);

    auto* titleLabel = new QLabel(title, button);
    titleLabel->setObjectName("actionTitle");
    titleLabel->setAlignment(Qt::AlignCenter);
    layout->addWidget(titleLabel);

    auto* descLabel = new QLabel(description, button);
    descLabel->setObjectName("actionDescription");
    descLabel->setAlignment(Qt::AlignCenter);
    descLabel->setWordWrap(true);
    layout->addWidget(descLabel);

    return button;
}

void MainContent::applyTheme() {
    const auto& colors = Theme::instance().colors();

    QString styleSheet = QString(
        "MainContent {"
        "    background-color: %1;"
        "}"
        "QLabel#titleLabel {"
        "    color: %2;"
        "    font-size: 20px;"
        "    font-weight: bold;"
        "}"
        "QLabel#subtitleLabel {"
        "    color: %3;"
        "    font-size: 14px;"
        "}"
        "QPushButton#quickActionButton {"
        "    background-color: %1;"
        "    border: 1px solid %4;"
        "    border-radius: %5px;"
        "}"
        "QPushButton#quickActionButton:hover {"
        "    background-color: %6;"
        "}"
        "QLabel#actionTitle {"
        "    color: %2;"
        "    font-size: 14px;"
        "    font-weight: bold;"
        "}"
        "QLabel#actionDescription {"
        "    color: %3;"
        "    font-size: 12px;"
        "}"
    ).arg(colors.background.name())
     .arg(colors.text.name())
     .arg(colors.textSecondary.name())
     .arg(colors.border.name())
     .arg(Theme::Sizes::borderRadius)
     .arg(colors.surface.name());

    setStyleSheet(styleSheet);
}

void MainContent::onThemeChanged() {
    applyTheme();
}