#pragma once

#include <QLineEdit>

class GLineEdit final : public QLineEdit {
    Q_OBJECT

  public:
    explicit GLineEdit(QWidget* parent = nullptr);
    explicit GLineEdit(const QString& text, QWidget* parent = nullptr);

  protected:
    void focusInEvent(QFocusEvent* event) override;

  private:
    void applyStyle();
};
