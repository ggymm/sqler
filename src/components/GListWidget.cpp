#include "GListWidget.h"

GListWidget::GListWidget(QWidget* parent) : QListWidget(parent)
{
    setFrameShape(QFrame::NoFrame);
    setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    setVerticalScrollMode(QAbstractItemView::ScrollPerPixel);
    setSelectionMode(QAbstractItemView::SingleSelection);
    applyStyle();
}

void GListWidget::applyStyle() {}
