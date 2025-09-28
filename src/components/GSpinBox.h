#pragma once

#include <QSpinBox>

class GSpinBox : public QSpinBox {
    Q_OBJECT

public:
    explicit GSpinBox(QWidget* parent = nullptr);

private:
    void applyStyle();
};

