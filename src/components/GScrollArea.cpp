#include "GScrollArea.h"

GScrollArea::GScrollArea(QWidget* parent) : QScrollArea(parent) {
    setWidgetResizable(true);
    applyStyle();
}

void GScrollArea::applyStyle() {}
