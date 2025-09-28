#include "MainContent.h"
#include "components/GLabel.h"
#include "components/GPushButton.h"
#include "components/GStyle.h"
#include <QGridLayout>
#include <QHBoxLayout>
#include <QIcon>
#include <QSpacerItem>
#include <QVBoxLayout>

MainContent::MainContent(QWidget* parent) : QWidget(parent) { setupUI(); }

void MainContent::setupUI()
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(GStyle::Spacing::lg, GStyle::Spacing::lg, GStyle::Spacing::lg, GStyle::Spacing::lg);
    layout->setSpacing(GStyle::Spacing::lg);

    layout->addStretch();

    m_titleLabel = new GLabel("欢迎使用 SQL 数据库管理器", GLabel::Role::Title, this);
    m_titleLabel->setAlignment(Qt::AlignCenter);
    layout->addWidget(m_titleLabel);

    m_subtitleLabel = new GLabel("请从左侧选择数据库连接开始使用", GLabel::Role::Subtitle, this);
    m_subtitleLabel->setAlignment(Qt::AlignCenter);
    layout->addWidget(m_subtitleLabel);

    layout->addSpacerItem(new QSpacerItem(0, GStyle::Spacing::xl, QSizePolicy::Minimum, QSizePolicy::Fixed));

    auto* actionsLayout = new QHBoxLayout();
    actionsLayout->setSpacing(GStyle::Spacing::lg);

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

GPushButton* MainContent::createQuickActionButton(const QString& title, const QString& description, const QString& iconPath)
{
    auto* button = new GPushButton(this);
    button->setVariant(GPushButton::Variant::Secondary);
    button->setFixedSize(200, 120);

    auto* layout = new QVBoxLayout(button);
    layout->setAlignment(Qt::AlignCenter);
    layout->setSpacing(GStyle::Spacing::sm);

    auto* iconLabel = new GLabel(button);
    iconLabel->setPixmap(QIcon(iconPath).pixmap(32, 32));
    iconLabel->setAlignment(Qt::AlignCenter);
    layout->addWidget(iconLabel);

    auto* titleLabel = new GLabel(title, GLabel::Role::Emphasis, button);
    titleLabel->setAlignment(Qt::AlignCenter);
    layout->addWidget(titleLabel);

    auto* descLabel = new GLabel(description, GLabel::Role::Caption, button);
    descLabel->setAlignment(Qt::AlignCenter);
    descLabel->setWordWrap(true);
    layout->addWidget(descLabel);

    return button;
}
