#pragma once

#include <QSpinBox>

class GSpinBox final : public QSpinBox {
    Q_OBJECT

  public:
    explicit GSpinBox(QWidget* parent = nullptr);

  private:
    void applyStyle();
};
