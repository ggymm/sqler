#pragma once

#include <QScrollArea>

class GScrollArea final : public QScrollArea {
    Q_OBJECT

  public:
    explicit GScrollArea(QWidget* parent = nullptr);

  private:
    void applyStyle();
};
