#include "TopMenuBar.h"
#include "Theme.h"
#include "dialogs/NewConnectionDialog.h"
#include <QHBoxLayout>
#include <QPushButton>
#include <QToolButton>
#include <QFrame>
#include <QSpacerItem>
#include <QIcon>

TopMenuBar::TopMenuBar(QWidget* parent)
    : QWidget(parent) {
    setupUI();
    applyTheme();

    connect(&Theme::instance(), &Theme::themeChanged, this, &TopMenuBar::onThemeChanged);
}

void TopMenuBar::setupUI() {
    auto* layout = new QHBoxLayout(this);
    layout->setContentsMargins(Theme::Spacing::sm, Theme::Spacing::sm, Theme::Spacing::sm, Theme::Spacing::sm);
    layout->setSpacing(Theme::Spacing::sm);

    m_newConnectionBtn = createMenuButton(QStringLiteral("新建连接"), QStringLiteral(":/assets/icons/new-conn.svg"));
    layout->addWidget(m_newConnectionBtn);
    connect(m_newConnectionBtn, &QPushButton::clicked, [this]() {
        auto* dialog = new NewConnectionDialog(this);
        dialog->exec();
        dialog->deleteLater();
    });

    m_newQueryBtn = createMenuButton(QStringLiteral("新建查询"), QStringLiteral(":/assets/icons/new-query.svg"));
    layout->addWidget(m_newQueryBtn);
    connect(m_newQueryBtn, &QPushButton::clicked, this, &TopMenuBar::newQueryClicked);

    auto* separator1 = new QFrame(this);
    separator1->setFrameShape(QFrame::VLine);
    separator1->setLineWidth(1);
    constexpr double separatorHeightRatio = 0.6;
    separator1->setFixedSize(1, static_cast<int>(Theme::Sizes::buttonHeight * separatorHeightRatio));
    layout->addWidget(separator1);

    m_tablesBtn = createMenuButton(QStringLiteral("表"), QStringLiteral(":/assets/icons/table.svg"));
    layout->addWidget(m_tablesBtn);
    connect(m_tablesBtn, &QPushButton::clicked, this, &TopMenuBar::tablesClicked);

    m_queryBtn = createMenuButton(QStringLiteral("查询"), QStringLiteral(":/assets/icons/query.svg"));
    layout->addWidget(m_queryBtn);
    connect(m_queryBtn, &QPushButton::clicked, this, &TopMenuBar::queryClicked);

    m_functionsBtn = createMenuButton(QStringLiteral("函数"), QStringLiteral(":/assets/icons/function.svg"));
    layout->addWidget(m_functionsBtn);
    connect(m_functionsBtn, &QPushButton::clicked, this, &TopMenuBar::functionsClicked);

    m_usersBtn = createMenuButton(QStringLiteral("用户"), QStringLiteral(":/assets/icons/user.svg"));
    layout->addWidget(m_usersBtn);
    connect(m_usersBtn, &QPushButton::clicked, this, &TopMenuBar::usersClicked);

    layout->addStretch();

    m_themeToggle = new QToolButton(this);
    m_themeToggle->setText(Theme::instance().isDarkMode() ? QStringLiteral("☀️") : QStringLiteral("🌙"));
    m_themeToggle->setFixedSize(Theme::Sizes::buttonHeight, Theme::Sizes::buttonHeight);
    layout->addWidget(m_themeToggle);
    connect(m_themeToggle, &QToolButton::clicked, this, &TopMenuBar::toggleTheme);
}

QPushButton* TopMenuBar::createMenuButton(const QString& text, const QString& iconPath) {
    auto* button = new QPushButton(text, this);
    button->setIcon(QIcon(iconPath));
    button->setIconSize(QSize(Theme::Sizes::iconSize, Theme::Sizes::iconSize));
    button->setFixedHeight(Theme::Sizes::buttonHeight);
    button->setFlat(true);
    return button;
}

void TopMenuBar::applyTheme() {
    const auto& colors = Theme::instance().colors();

    const QString buttonStyle = QStringLiteral(
        "QPushButton {"
        "    color: %1;"
        "    background-color: transparent;"
        "    border: none;"
        "    padding: 4px 8px;"
        "    border-radius: %2px;"
        "    font-size: 14px;"
        "}"
        "QPushButton:hover {"
        "    background-color: %3;"
        "}"
        "QFrame[frameShape=\"5\"] {"
        "    color: %4;"
        "}"
        "QToolButton {"
        "    color: %1;"
        "    background-color: transparent;"
        "    border: none;"
        "    border-radius: %2px;"
        "    font-size: 16px;"
        "}"
        "QToolButton:hover {"
        "    background-color: %3;"
        "}"
    ).arg(colors.text.name())
     .arg(Theme::Sizes::borderRadius)
     .arg(colors.border.name())
     .arg(colors.border.name());

    setStyleSheet(QStringLiteral("TopMenuBar { background-color: %1; border-bottom: 1px solid %2; }")
                  .arg(colors.surface.name(), colors.border.name()) + buttonStyle);

    m_themeToggle->setText(Theme::instance().isDarkMode() ? QStringLiteral("☀️") : QStringLiteral("🌙"));
}

void TopMenuBar::onThemeChanged() {
    applyTheme();
}

void TopMenuBar::toggleTheme() {
    Theme::instance().setDarkMode(!Theme::instance().isDarkMode());
}