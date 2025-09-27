#pragma once

#include <QWidget>

class QLabel;
class QPushButton;

class MainContent : public QWidget {
    Q_OBJECT

public:
    explicit MainContent(QWidget* parent = nullptr);

private slots:
    void onThemeChanged();

private:
    void setupUI();
    void applyTheme();
    QPushButton* createQuickActionButton(const QString& title, const QString& description, const QString& iconPath);

    QLabel* m_titleLabel;
    QLabel* m_subtitleLabel;
    QPushButton* m_newConnectionAction;
    QPushButton* m_newQueryAction;
    QPushButton* m_browseTablesAction;
};