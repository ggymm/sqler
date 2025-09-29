#include "GLineEdit.h"
#include <QFocusEvent>

GLineEdit::GLineEdit(QWidget* parent) : QLineEdit(parent) {}

GLineEdit::GLineEdit(const QString& text, QWidget* parent) : QLineEdit(text, parent) {}

void GLineEdit::focusInEvent(QFocusEvent* event) {
    QLineEdit::focusInEvent(event);
    // Avoid selecting all text when focus is gained programmatically/tabbing
    if (event && event->reason() != Qt::MouseFocusReason) {
        deselect();
        setCursorPosition(text().length());
    }
}

void GLineEdit::applyStyle() {}
