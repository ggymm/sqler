#pragma once

#include "../components/ThemedDialog.h"

class ThemedScrollArea;
class ThemedButton;

struct DatabaseType {
    QString id;
    QString displayName;
    QString description;
    QString iconPath;
};

class DatabaseTypeDialog : public ThemedDialog {
    Q_OBJECT

public:
    explicit DatabaseTypeDialog(QWidget* parent = nullptr);

    [[nodiscard]] QString selectedDatabaseType() const { return m_selectedType; }

private slots:
    void onDatabaseTypeSelected(const QString& type);

private:
    void setupUI();
    ThemedButton* createDatabaseTypeButton(const DatabaseType& dbType);

    QString m_selectedType;
    QList<DatabaseType> m_databaseTypes;
};