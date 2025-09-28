#pragma once

#include <QWidget>

class GListWidget;
class QListWidgetItem;
class GPushButton;

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

    GListWidget* m_connectionsList;
    GPushButton* m_addButton;
};
