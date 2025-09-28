#pragma once

#include "ConnectionFormBase.h"

class GLineEdit;
class GPushButton;

class SQLiteConnectionForm : public ConnectionFormBase {
    Q_OBJECT

public:
    explicit SQLiteConnectionForm(QWidget* parent = nullptr);

    QVariantMap getConnectionData() const override;
    bool validateInput() const override;

protected:
    void setupUI() override;

private slots:
    void browseFile();
    void createNewFile();

private:
    GLineEdit* m_nameEdit;
    GLineEdit* m_filePathEdit;
    GPushButton* m_browseButton;
    GPushButton* m_newFileButton;
};
