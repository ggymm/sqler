#include "TopMenuBar.h"

#include "components/GPushButton.h"
#include "components/GSeparator.h"
#include "components/GStyle.h"
#include "dialogs/NewConnectionDialog.h"

#include <QHBoxLayout>

TopMenuBar::TopMenuBar(QWidget* parent) : QWidget(parent)
{
    setupUI();
}

void TopMenuBar::setupUI()
{
    auto* layout = new QHBoxLayout(this);
    layout->setContentsMargins(GStyle::Spacing::sm, GStyle::Spacing::sm, GStyle::Spacing::sm, GStyle::Spacing::sm);
    layout->setSpacing(GStyle::Spacing::sm);

    m_newConnectionBtn = createMenuButton(QStringLiteral("新建连接"), QStringLiteral(":/assets/icons/new-conn.svg"));
    layout->addWidget(m_newConnectionBtn);
    connect(m_newConnectionBtn, &QPushButton::clicked,
            [this]()
            {
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
}

GPushButton* TopMenuBar::createMenuButton(const QString& text, const QString& iconPath)
{
    auto* button = new GPushButton(text, GPushButton::Variant::Toolbar, this);
    button->setIcon(QIcon(iconPath));
    button->setIconSize(QSize(GStyle::Sizes::iconSize, GStyle::Sizes::iconSize));
    button->setFixedHeight(GStyle::Sizes::buttonHeight);
    return button;
}
