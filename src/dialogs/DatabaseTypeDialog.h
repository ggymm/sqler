#pragma once

#include <QDialog>

class QGridLayout;
class QPushButton;

struct DatabaseType {
    QString id;
    QString displayName;
    QString description;
    QString iconPath;
};

class DatabaseTypeDialog : public QDialog {
    Q_OBJECT

public:
    explicit DatabaseTypeDialog(QWidget* parent = nullptr);

    QString selectedDatabaseType() const { return m_selectedType; }

private slots:
    void onThemeChanged();
    void onDatabaseTypeSelected(const QString& type);

private:
    void setupUI();
    void applyTheme();
    QPushButton* createDatabaseTypeButton(const DatabaseType& dbType);

    QGridLayout* m_gridLayout;
    QString m_selectedType;
    QList<DatabaseType> m_databaseTypes;
};