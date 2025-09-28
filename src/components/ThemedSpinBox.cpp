#include "ThemedSpinBox.h"
#include "../Theme.h"

ThemedSpinBox::ThemedSpinBox(QWidget* parent)
    : QSpinBox(parent) {
    setup();
}

void ThemedSpinBox::setup() {
    applyTheme();
    connect(&Theme::instance(), &Theme::themeChanged, this, &ThemedSpinBox::onThemeChanged);
}

void ThemedSpinBox::onThemeChanged() {
    applyTheme();
}

void ThemedSpinBox::applyTheme() {
    setStyleSheet(Theme::instance().getInputStyle());
}

