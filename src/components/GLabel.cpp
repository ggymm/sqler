#include "GLabel.h"
#include <QStyle>

static QString roleToString(GLabel::Role r)
{
    switch (r)
    {
    case GLabel::Role::Title:
        return "title";
    case GLabel::Role::Subtitle:
        return "subtitle";
    case GLabel::Role::Body:
        return "body";
    case GLabel::Role::Caption:
        return "caption";
    case GLabel::Role::Emphasis:
        return "emphasis";
    }
    return "body";
}

GLabel::GLabel(QWidget* parent) : QLabel(parent) { setup(); }
GLabel::GLabel(const QString& text, QWidget* parent) : QLabel(text, parent) { setup(); }
GLabel::GLabel(const QString& text, Role role, QWidget* parent) : QLabel(text, parent), m_role(role) { setup(); }

void GLabel::setup()
{
    setWordWrap(false);
    applyStyle();
}

void GLabel::setRole(Role role)
{
    if (m_role != role)
    {
        m_role = role;
        applyStyle();
    }
}

void GLabel::applyStyle()
{
    setProperty("gRole", roleToString(m_role));
    style()->unpolish(this);
    style()->polish(this);
    update();
}
