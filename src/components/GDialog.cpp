#include "GDialog.h"

GDialog::GDialog(QWidget* parent) : QDialog(parent) {
    setModal(true);
    setAttribute(Qt::WA_StyledBackground, true);
    applyStyle();
}

void GDialog::applyStyle() {}
