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

    m_newConnectionBtn = createMenuButton("新建连接", ":/assets/icons/new-conn.svg");
    layout->addWidget(m_newConnectionBtn);
    connect(m_newConnectionBtn, &QPushButton::clicked, [this]() {
        auto* dialog = new NewConnectionDialog(this);
        dialog->exec();
        dialog->deleteLater();
    });

    m_newQueryBtn = createMenuButton("新建查询", ":/assets/icons/new-query.svg");
    layout->addWidget(m_newQueryBtn);
    connect(m_newQueryBtn, &QPushButton::clicked, this, &TopMenuBar::newQueryClicked);

    auto* separator1 = new QFrame(this);
    separator1->setFrameShape(QFrame::VLine);
    separator1->setLineWidth(1);
    separator1->setFixedSize(1, Theme::Sizes::buttonHeight * 0.6);
    layout->addWidget(separator1);

    m_tablesBtn = createMenuButton("表", ":/assets/icons/table.svg");
    layout->addWidget(m_tablesBtn);
    connect(m_tablesBtn, &QPushButton::clicked, this, &TopMenuBar::tablesClicked);

    m_queryBtn = createMenuButton("查询", ":/assets/icons/query.svg");
    layout->addWidget(m_queryBtn);
    connect(m_queryBtn, &QPushButton::clicked, this, &TopMenuBar::queryClicked);

    m_functionsBtn = createMenuButton("函数", ":/assets/icons/function.svg");
    layout->addWidget(m_functionsBtn);
    connect(m_functionsBtn, &QPushButton::clicked, this, &TopMenuBar::functionsClicked);

    m_usersBtn = createMenuButton("用户", ":/assets/icons/user.svg");
    layout->addWidget(m_usersBtn);
    connect(m_usersBtn, &QPushButton::clicked, this, &TopMenuBar::usersClicked);

    layout->addStretch();

    m_themeToggle = new QToolButton(this);
    m_themeToggle->setText(Theme::instance().isDarkMode() ? "☀️" : "🌙");
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

    QString buttonStyle = QString(
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

    setStyleSheet(QString("TopMenuBar { background-color: %1; border-bottom: 1px solid %2; }")
                  .arg(colors.surface.name(), colors.border.name()) + buttonStyle);

    m_themeToggle->setText(Theme::instance().isDarkMode() ? "☀️" : "🌙");
}

void TopMenuBar::onThemeChanged() {
    applyTheme();
}

void TopMenuBar::toggleTheme() {
    Theme::instance().setDarkMode(!Theme::instance().isDarkMode());
}