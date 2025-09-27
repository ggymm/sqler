#include "ThemedScrollArea.h"
#include "../Theme.h"

ThemedScrollArea::ThemedScrollArea(QWidget* parent)
    : QScrollArea(parent) {
    setupScrollArea();
}

void ThemedScrollArea::setupScrollArea() {
    setWidgetResizable(true);
    setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    setVerticalScrollBarPolicy(Qt::ScrollBarAsNeeded);
    setFrameShape(QFrame::NoFrame);

    applyTheme();
    connect(&Theme::instance(), &Theme::themeChanged, this, &ThemedScrollArea::onThemeChanged);
}

void ThemedScrollArea::onThemeChanged() {
    applyTheme();
}

void ThemedScrollArea::applyTheme() {
    setStyleSheet(Theme::instance().getScrollAreaStyle());
}