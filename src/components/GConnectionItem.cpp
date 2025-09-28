#include "GConnectionItem.h"
#include "GLabel.h"
#include "GStyle.h"
#include <QHBoxLayout>
#include <QIcon>
#include <QStyle>
#include <QVBoxLayout>

GConnectionItem::GConnectionItem(const QString& name, const QString& type, bool connected, QWidget* parent) : QWidget(parent), m_connected(connected)
{
    buildUI(name, type);
    applyStyle();
}

void GConnectionItem::buildUI(const QString& name, const QString& type)
{
    using namespace GStyle;
    auto* layout = new QHBoxLayout(this);
    layout->setContentsMargins(Spacing::sm, Spacing::sm, Spacing::sm, Spacing::sm);
    layout->setSpacing(Spacing::sm);

    auto* iconLabel = new GLabel(this);
    iconLabel->setFixedSize(24, 24);
    iconLabel->setPixmap(QIcon(QStringLiteral(":/assets/icons/db/%1.svg").arg(type)).pixmap(20, 20));
    iconLabel->setAlignment(Qt::AlignCenter);
    layout->addWidget(iconLabel);

    auto* textLayout = new QVBoxLayout();
    textLayout->setSpacing(2);
    textLayout->setContentsMargins(0, 0, 0, 0);
    m_nameLabel = new GLabel(name, GLabel::Role::Body, this);
    textLayout->addWidget(m_nameLabel);
    m_typeLabel = new GLabel(type.toUpper(), GLabel::Role::Caption, this);
    textLayout->addWidget(m_typeLabel);
    layout->addLayout(textLayout);

    layout->addStretch();

    m_statusDot = new GLabel("●", GLabel::Role::Caption, this);
    layout->addWidget(m_statusDot);
}

void GConnectionItem::applyStyle()
{
    setProperty("gSelected", m_selected ? "true" : "false");
    m_statusDot->setProperty("gStatus", m_connected ? "connected" : "disconnected");
    style()->unpolish(this);
    style()->polish(this);
    update();
    m_statusDot->style()->unpolish(m_statusDot);
    m_statusDot->style()->polish(m_statusDot);
    m_statusDot->update();
}

void GConnectionItem::setSelected(bool sel)
{
    if (m_selected != sel)
    {
        m_selected = sel;
        applyStyle();
    }
}

void GConnectionItem::setConnected(bool connected)
{
    if (m_connected != connected)
    {
        m_connected = connected;
        applyStyle();
    }
}
