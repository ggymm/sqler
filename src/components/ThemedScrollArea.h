#pragma once

#include <QScrollArea>

class ThemedScrollArea : public QScrollArea {
    Q_OBJECT

public:
    explicit ThemedScrollArea(QWidget* parent = nullptr);

private slots:
    void onThemeChanged();

private:
    void setupScrollArea();
    void applyTheme();
};