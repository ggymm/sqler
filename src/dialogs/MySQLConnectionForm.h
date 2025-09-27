#pragma once

#include "ConnectionFormBase.h"

class QLineEdit;
class QSpinBox;

class MySQLConnectionForm : public ConnectionFormBase {
    Q_OBJECT

public:
    explicit MySQLConnectionForm(QWidget* parent = nullptr);

    QVariantMap getConnectionData() const override;
    bool validateInput() const override;

protected:
    void setupUI() override;

private:
    QLineEdit* m_nameEdit;
    QLineEdit* m_hostEdit;
    QSpinBox* m_portSpin;
    QLineEdit* m_usernameEdit;
    QLineEdit* m_passwordEdit;
    QLineEdit* m_databaseEdit;
};