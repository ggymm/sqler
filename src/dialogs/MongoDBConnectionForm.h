#pragma once

#include "ConnectionFormBase.h"

class GLineEdit;
class GSpinBox;

class MongoDBConnectionForm : public ConnectionFormBase {
    Q_OBJECT

public:
    explicit MongoDBConnectionForm(QWidget* parent = nullptr);

    QVariantMap getConnectionData() const override;
    bool validateInput() const override;

protected:
    void setupUI() override;

private:
    GLineEdit* m_nameEdit;
    GLineEdit* m_connectionStringEdit;
    GLineEdit* m_hostEdit;
    GSpinBox* m_portSpin;
    GLineEdit* m_usernameEdit;
    GLineEdit* m_passwordEdit;
    GLineEdit* m_databaseEdit;
};
