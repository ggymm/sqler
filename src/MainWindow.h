#pragma once

#include <QMainWindow>

class TopMenuBar;
class ConnectionPanel;
class MainContent;

class MainWindow : public QMainWindow {
    Q_OBJECT

public:
    explicit MainWindow(QWidget* parent = nullptr);

private slots:
    void onThemeChanged();

private:
    void setupUI();
    void applyTheme();

    TopMenuBar* m_topMenuBar;
    ConnectionPanel* m_connectionPanel;
    MainContent* m_mainContent;
};