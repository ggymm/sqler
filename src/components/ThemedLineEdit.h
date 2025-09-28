#pragma once

#include <QLineEdit>

class ThemedLineEdit : public QLineEdit {
    Q_OBJECT

public:
    explicit ThemedLineEdit(QWidget* parent = nullptr);
    explicit ThemedLineEdit(const QString& text, QWidget* parent = nullptr);

private slots:
    void onThemeChanged();

private:
    void setup();
    void applyTheme();
};

