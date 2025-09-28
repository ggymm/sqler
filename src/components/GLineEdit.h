#pragma once

#include <QLineEdit>

class GLineEdit : public QLineEdit {
    Q_OBJECT

public:
    explicit GLineEdit(QWidget* parent = nullptr);
    explicit GLineEdit(const QString& text, QWidget* parent = nullptr);

private:
    void applyStyle();
};

