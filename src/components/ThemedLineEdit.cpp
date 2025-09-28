#include "ThemedLineEdit.h"
#include "../Theme.h"

ThemedLineEdit::ThemedLineEdit(QWidget* parent)
    : QLineEdit(parent) {
    setup();
}

ThemedLineEdit::ThemedLineEdit(const QString& text, QWidget* parent)
    : QLineEdit(text, parent) {
    setup();
}

void ThemedLineEdit::setup() {
    applyTheme();
    connect(&Theme::instance(), &Theme::themeChanged, this, &ThemedLineEdit::onThemeChanged);
}

void ThemedLineEdit::onThemeChanged() {
    applyTheme();
}

void ThemedLineEdit::applyTheme() {
    // Reuse centralized input style (covers QLineEdit, QSpinBox, etc.)
    setStyleSheet(Theme::instance().getInputStyle());
}

