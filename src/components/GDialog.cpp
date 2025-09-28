#include "GDialog.h"

GDialog::GDialog(QWidget* parent) : QDialog(parent) {
    setModal(true);
    applyStyle();
}

void GDialog::applyStyle() {}
