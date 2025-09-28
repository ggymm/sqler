#pragma once

#include <QScrollArea>

class GScrollArea : public QScrollArea
{
    Q_OBJECT

  public:
    explicit GScrollArea(QWidget* parent = nullptr);

  private:
    void applyStyle();
};
