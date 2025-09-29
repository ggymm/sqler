#include "TopMenuBar.h"

#include "components/GPushButton.h"
#include "components/GSeparator.h"
#include "components/GStyle.h"
#include "components/GTheme.h"
#include "dialogs/NewConnectionDialog.h"

#include <QHBoxLayout>

#include <algorithm>

TopMenuBar::TopMenuBar(QWidget* parent) : QWidget(parent) {
    setupUI();
    updateThemeIcon();

    // Connect to theme changes to update icon
    connect(&GTheme::instance(), &GTheme::themeChanged, this, &TopMenuBar::updateThemeIcon);
}

void TopMenuBar::setupUI() {
    auto* layout = new QHBoxLayout(this);
    // Compute vertical margins so toolbar buttons (fixed height)
    // are vertically centered within the top bar height.
    const int vMargin = std::max(0, (GStyle::Sizes::topBarHeight - GStyle::Sizes::buttonHeight) / 2);
    layout->setContentsMargins(GStyle::Spacing::sm, vMargin, GStyle::Spacing::sm, vMargin);
    layout->setSpacing(GStyle::Spacing::sm);

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

    auto* separator1 = new GSeparator(GSeparator::Orientation::Vertical, this);
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

    // Add theme toggle button at the right end
    m_themeToggleBtn = new GPushButton(QString(), GPushButton::Variant::Toolbar, this);
    m_themeToggleBtn->setIconSize(QSize(GStyle::Sizes::iconSize, GStyle::Sizes::iconSize));
    m_themeToggleBtn->setFixedHeight(GStyle::Sizes::buttonHeight);
    m_themeToggleBtn->setToolTip(QStringLiteral("切换主题"));
    layout->addWidget(m_themeToggleBtn);
    connect(m_themeToggleBtn, &QPushButton::clicked, this, &TopMenuBar::themeToggleClicked);
}

GPushButton* TopMenuBar::createMenuButton(const QString& text, const QString& iconPath) {
    auto* button = new GPushButton(text, GPushButton::Variant::Toolbar, this);
    button->setIcon(QIcon(iconPath));
    button->setIconSize(QSize(GStyle::Sizes::iconSize, GStyle::Sizes::iconSize));
    button->setFixedHeight(GStyle::Sizes::buttonHeight);
    return button;
}

void TopMenuBar::updateThemeIcon() {
    if (!m_themeToggleBtn)
        return;

    const auto& theme = GTheme::instance();
    const QString iconPath =
        theme.mode() == GTheme::Mode::Light ? QStringLiteral(":/assets/icons/theme-dark.svg") : QStringLiteral(":/assets/icons/theme-light.svg");

    m_themeToggleBtn->setIcon(QIcon(iconPath));
}
