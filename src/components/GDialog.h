#pragma once

#include <QDialog>

class GDialog : public QDialog
{
    Q_OBJECT

  public:
    explicit GDialog(QWidget* parent = nullptr);

  private:
    void applyStyle();
};
