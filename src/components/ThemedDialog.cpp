#include "ThemedDialog.h"
#include "../Theme.h"

ThemedDialog::ThemedDialog(QWidget* parent)
    : QDialog(parent) {
    setupDialog();
}

void ThemedDialog::setupDialog() {
    setModal(true);
    applyTheme();
    connect(&Theme::instance(), &Theme::themeChanged, this, &ThemedDialog::onThemeChanged);
}

void ThemedDialog::onThemeChanged() {
    applyTheme();
}

void ThemedDialog::applyTheme() {
    setStyleSheet(Theme::instance().getDialogStyle());
}