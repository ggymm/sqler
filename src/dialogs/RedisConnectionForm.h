#pragma once

#include "ConnectionFormBase.h"

class GLineEdit;
class GSpinBox;

class RedisConnectionForm : public ConnectionFormBase
{
    Q_OBJECT

  public:
    explicit RedisConnectionForm(QWidget* parent = nullptr);

    QVariantMap getConnectionData() const override;
    bool validateInput() const override;

  protected:
    void setupUI() override;

  private:
    GLineEdit* m_nameEdit{};
    GLineEdit* m_hostEdit{};
    GSpinBox* m_portSpin{};
    GLineEdit* m_usernameEdit{};
    GLineEdit* m_passwordEdit{};
    GSpinBox* m_databaseSpin{};
};
