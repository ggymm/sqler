#pragma once

#include <QWidget>

class QListWidget;
class QListWidgetItem;
class QPushButton;

class ConnectionPanel : public QWidget {
    Q_OBJECT

public:
    explicit ConnectionPanel(QWidget* parent = nullptr);

signals:
    void connectionSelected(const QString& connectionName);

private slots:
    void onThemeChanged();
    void onConnectionItemClicked(QListWidgetItem* item);

private:
    void setupUI();
    void applyTheme();
    void populateConnections();
    QListWidgetItem* createConnectionItem(const QString& name, const QString& type, bool connected);

    QListWidget* m_connectionsList;
    QPushButton* m_addButton;
};