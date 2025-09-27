#include "ThemedButton.h"
#include "../Theme.h"

ThemedButton::ThemedButton(QWidget* parent)
    : QPushButton(parent) {
    setupButton();
}

ThemedButton::ThemedButton(const QString& text, QWidget* parent)
    : QPushButton(text, parent) {
    setupButton();
}

ThemedButton::ThemedButton(const QString& text, Variant variant, QWidget* parent)
    : QPushButton(text, parent)
    , m_variant(variant) {
    setupButton();
}

void ThemedButton::setupButton() {
    applyTheme();
    connect(&Theme::instance(), &Theme::themeChanged, this, &ThemedButton::onThemeChanged);
}

void ThemedButton::setVariant(Variant variant) {
    if (m_variant != variant) {
        m_variant = variant;
        applyTheme();
    }
}

void ThemedButton::onThemeChanged() {
    applyTheme();
}

void ThemedButton::applyTheme() {
    QString variantName;
    switch (m_variant) {
        case Variant::Primary:  variantName = "primary"; break;
        case Variant::Secondary: variantName = "secondary"; break;
        case Variant::Dialog:   variantName = "dialog"; break;
    }

    setStyleSheet(Theme::instance().getButtonStyle(variantName));
}