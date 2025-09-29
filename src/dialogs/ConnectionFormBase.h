#pragma once

#include <QVariantMap>
#include <QWidget>

class QFormLayout;
class GPushButton;
class QHBoxLayout;
class QWidget;

class ConnectionFormBase : public QWidget {
    Q_OBJECT

  public:
    explicit ConnectionFormBase(QWidget* parent = nullptr);

    virtual QVariantMap getConnectionData() const = 0;
    virtual bool validateInput() const = 0;

    // Attach this form's footer buttons into an external footer layout
    void attachFooterTo(QHBoxLayout* destLayout);

  signals:
    void connectionSaved();
    void backClicked();
    void cancelClicked();

  protected slots:
    virtual void onTestConnection();
    virtual void onSaveConnection();

  protected:
    virtual void setupUI() = 0;
    void showEvent(QShowEvent* event) override;

    QFormLayout* m_formLayout;
    GPushButton* m_testButton;
    GPushButton* m_backButton;
    GPushButton* m_cancelButton;
    GPushButton* m_saveButton;
    QWidget* m_footerWidget{};
};
