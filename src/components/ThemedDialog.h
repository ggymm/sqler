#pragma once

#include <QDialog>

class ThemedDialog : public QDialog {
    Q_OBJECT

public:
    explicit ThemedDialog(QWidget* parent = nullptr);

protected:
    void setupDialog();

private slots:
    void onThemeChanged();

private:
    void applyTheme();
};