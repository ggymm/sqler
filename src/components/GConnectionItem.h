#pragma once

#include <QWidget>

class GLabel;

class GConnectionItem : public QWidget {
    Q_OBJECT

public:
    explicit GConnectionItem(const QString& name,
                              const QString& type,
                              bool connected,
                              QWidget* parent = nullptr);

    void setSelected(bool sel);
    void setConnected(bool connected);

private:
    void buildUI(const QString& name, const QString& type);
    void applyStyle();

    bool m_selected = false;
    bool m_connected = false;
    GLabel* m_nameLabel = nullptr;
    GLabel* m_typeLabel = nullptr;
    GLabel* m_statusDot = nullptr;
};

