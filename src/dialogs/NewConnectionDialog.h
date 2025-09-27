#pragma once

#include <QDialog>

class QStackedWidget;
class QPushButton;
class QHBoxLayout;
class DatabaseTypeDialog;
class ConnectionFormBase;

class NewConnectionDialog : public QDialog {
    Q_OBJECT

public:
    explicit NewConnectionDialog(QWidget* parent = nullptr);

private slots:
    void onThemeChanged();
    void showDatabaseTypeSelection();
    void showConnectionForm(const QString& databaseType);
    void onBackClicked();
    void onConnectionSaved();

private:
    void setupUI();
    void applyTheme();
    ConnectionFormBase* createConnectionForm(const QString& databaseType);

    QStackedWidget* m_stackedWidget;
    QPushButton* m_backButton;
    QPushButton* m_cancelButton;
    QHBoxLayout* m_buttonLayout;

    DatabaseTypeDialog* m_typeDialog;
    ConnectionFormBase* m_currentForm;
    QString m_currentDatabaseType;
};