#include "GPushButton.h"

#include "GStyle.h"

#include <QStyle>

static QString variantToString(GPushButton::Variant v) {
    switch (v) {
    case GPushButton::Variant::Primary:
        return "primary";
    case GPushButton::Variant::Secondary:
        return "secondary";
    case GPushButton::Variant::Dialog:
        return "dialog";
    case GPushButton::Variant::Toolbar:
        return "toolbar";
    case GPushButton::Variant::Neutral:
        return "neutral";
    }
    return "neutral";
}

GPushButton::GPushButton(QWidget* parent) : QPushButton(parent) { setup(); }

GPushButton::GPushButton(const QString& text, QWidget* parent) : QPushButton(text, parent) { setup(); }

GPushButton::GPushButton(const QString& text, Variant variant, QWidget* parent) : QPushButton(text, parent), m_variant(variant) { setup(); }

void GPushButton::setup() {
    setCursor(Qt::PointingHandCursor);
    applyStyle();
}

void GPushButton::setVariant(Variant variant) {
    if (m_variant != variant) {
        m_variant = variant;
        applyStyle();
    }
}

void GPushButton::applyStyle() {
    setProperty("gVariant", variantToString(m_variant));
    style()->unpolish(this);
    style()->polish(this);
    update();
    if (m_variant == Variant::Toolbar)
        setFixedHeight(GStyle::Sizes::buttonHeight);
}
