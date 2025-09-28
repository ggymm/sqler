#include "ConnectionPanel.h"
#include "components/GConnectionItem.h"
#include "components/GLabel.h"
#include "components/GListWidget.h"
#include "components/GPushButton.h"
#include "components/GStyle.h"
#include <QHBoxLayout>
#include <QIcon>
#include <QListWidget>
#include <QListWidgetItem>
#include <QVBoxLayout>

ConnectionPanel::ConnectionPanel(QWidget* parent) : QWidget(parent)
{
    setAttribute(Qt::WA_StyledBackground, true);
    setObjectName("connectionPanel");
    setupUI();
    populateConnections();
}

void ConnectionPanel::setupUI()
{
    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(GStyle::Spacing::md, GStyle::Spacing::md, GStyle::Spacing::md, GStyle::Spacing::md);
    layout->setSpacing(GStyle::Spacing::md);

    auto* headerLayout = new QHBoxLayout();

    auto* titleLabel = new GLabel("连接管理", GLabel::Role::Emphasis, this);
    headerLayout->addWidget(titleLabel);

    headerLayout->addStretch();

    m_addButton = new GPushButton("+", GPushButton::Variant::Secondary, this);
    m_addButton->setFixedSize(32, 32);
    headerLayout->addWidget(m_addButton);

    layout->addLayout(headerLayout);

    m_connectionsList = new GListWidget(this);
    layout->addWidget(m_connectionsList);

    connect(m_connectionsList, &QListWidget::itemClicked, this, &ConnectionPanel::onConnectionItemClicked);
}

void ConnectionPanel::populateConnections()
{
    createConnectionItem("本地 MySQL", "mysql", true);
    createConnectionItem("测试 PostgreSQL", "postgresql", false);
    createConnectionItem("Redis 缓存", "redis", true);
}

QListWidgetItem* ConnectionPanel::createConnectionItem(const QString& name, const QString& type, bool connected)
{
    auto* item = new QListWidgetItem(m_connectionsList);
    auto* widget = new GConnectionItem(name, type, connected);
    item->setSizeHint(QSize(0, 48));
    m_connectionsList->setItemWidget(item, widget);
    return item;
}

void ConnectionPanel::onConnectionItemClicked(QListWidgetItem* item)
{
    if (!item)
        return;

    // Get the connection widget
    QWidget* connectionWidget = m_connectionsList->itemWidget(item);
    if (!connectionWidget)
        return;

    // Highlight the selected item using component API
    for (int i = 0; i < m_connectionsList->count(); ++i)
    {
        QListWidgetItem* listItem = m_connectionsList->item(i);
        if (auto* widget = qobject_cast<GConnectionItem*>(m_connectionsList->itemWidget(listItem)))
        {
            widget->setSelected(widget == connectionWidget);
        }
    }

    // Emit signal that a connection was selected
    emit connectionSelected(item->text());
}
