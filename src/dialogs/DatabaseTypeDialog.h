#pragma once

#include <QWidget>

class GScrollArea;
class GPushButton;
class GLabel;

struct DatabaseType
{
    QString id;
    QString displayName;
    QString description;
    QString iconPath;
};

class DatabaseTypeDialog : public QWidget
{
    Q_OBJECT

  public:
    explicit DatabaseTypeDialog(QWidget* parent = nullptr);

    [[nodiscard]] QString selectedDatabaseType() const { return m_selectedType; }

  private slots:
    void onDatabaseTypeSelected(const QString& type);

  private:
    void setupUI();
    GPushButton* createDatabaseTypeButton(const DatabaseType& dbType);

    QString m_selectedType;
    QList<DatabaseType> m_databaseTypes;

  signals:
    void selected(const QString& type);
    void cancelled();
};
