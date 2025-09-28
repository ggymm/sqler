#pragma once

#include <QListWidget>

class GListWidget : public QListWidget {
    Q_OBJECT

public:
    explicit GListWidget(QWidget* parent = nullptr);

private:
    void applyStyle();
};

