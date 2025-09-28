#pragma once

#include <QWidget>
#include <QVariantMap>

class QFormLayout;
class GPushButton;

class ConnectionFormBase : public QWidget {
    Q_OBJECT

public:
    explicit ConnectionFormBase(QWidget* parent = nullptr);

    virtual QVariantMap getConnectionData() const = 0;
    virtual bool validateInput() const = 0;

signals:
    void connectionSaved();
    void backClicked();
    void cancelClicked();

protected slots:
    virtual void onTestConnection();
    virtual void onSaveConnection();

protected:
    virtual void setupUI() = 0;

    QFormLayout* m_formLayout;
    GPushButton* m_testButton;
    GPushButton* m_backButton;
    GPushButton* m_cancelButton;
    GPushButton* m_saveButton;
};
