#pragma once

#include <QSpinBox>

class ThemedSpinBox : public QSpinBox {
    Q_OBJECT

public:
    explicit ThemedSpinBox(QWidget* parent = nullptr);

private slots:
    void onThemeChanged();

private:
    void setup();
    void applyTheme();
};

