#pragma once

#include <QListWidget>

class GListWidget final : public QListWidget
{
    Q_OBJECT

  public:
    explicit GListWidget(QWidget* parent = nullptr);

  private:
    void applyStyle();
};
