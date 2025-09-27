#pragma once

#include <QWidget>
#include <QVariantMap>

class QFormLayout;
class QLineEdit;
class QSpinBox;
class QPushButton;

class ConnectionFormBase : public QWidget {
    Q_OBJECT

public:
    explicit ConnectionFormBase(QWidget* parent = nullptr);

    virtual QVariantMap getConnectionData() const = 0;
    virtual bool validateInput() const = 0;

signals:
    void connectionSaved();

protected slots:
    virtual void onThemeChanged();
    virtual void onTestConnection();
    virtual void onSaveConnection();

protected:
    virtual void setupUI() = 0;
    virtual void applyTheme();

    QFormLayout* m_formLayout;
    QPushButton* m_testButton;
    QPushButton* m_saveButton;
};