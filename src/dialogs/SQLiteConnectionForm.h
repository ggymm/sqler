#pragma once

#include "ConnectionFormBase.h"

class QLineEdit;
class QPushButton;

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
    QLineEdit* m_nameEdit;
    QLineEdit* m_filePathEdit;
    QPushButton* m_browseButton;
    QPushButton* m_newFileButton;
};