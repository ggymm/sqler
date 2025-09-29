#pragma once

#include "../components/GDialog.h"

class QStackedWidget;
class GPushButton;
class QHBoxLayout;
class QWidget;
class DatabaseTypeDialog;
class ConnectionFormBase;

class NewConnectionDialog final : public GDialog {
    Q_OBJECT

  public:
    explicit NewConnectionDialog(QWidget* parent = nullptr);

  private slots:
    void showDatabaseTypeSelection();
    void showConnectionForm(const QString& databaseType);
    void onBackClicked();
    void onConnectionSaved();

  private:
    void setupUI();
    ConnectionFormBase* createConnectionForm(const QString& databaseType);

    QStackedWidget* m_stackedWidget;
    GPushButton* m_backButton;
    GPushButton* m_cancelButton;
    QHBoxLayout* m_buttonLayout;
    QWidget* m_footerWidget;

    DatabaseTypeDialog* m_typeDialog;
    ConnectionFormBase* m_currentForm;
    QString m_currentDatabaseType;
};
