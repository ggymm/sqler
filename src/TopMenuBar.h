#pragma once

#include <QWidget>

class QPushButton;
class QToolButton;

class TopMenuBar : public QWidget {
    Q_OBJECT

public:
    explicit TopMenuBar(QWidget* parent = nullptr);

signals:
    void newConnectionClicked();
    void newQueryClicked();
    void tablesClicked();
    void queryClicked();
    void functionsClicked();
    void usersClicked();

private slots:
    void onThemeChanged();
    void toggleTheme();

private:
    void setupUI();
    void applyTheme();
    QPushButton* createMenuButton(const QString& text, const QString& iconPath);

    QPushButton* m_newConnectionBtn;
    QPushButton* m_newQueryBtn;
    QPushButton* m_tablesBtn;
    QPushButton* m_queryBtn;
    QPushButton* m_functionsBtn;
    QPushButton* m_usersBtn;
    QToolButton* m_themeToggle;
};