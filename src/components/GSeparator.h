#pragma once

#include <QFrame>

class GSeparator : public QFrame
{
    Q_OBJECT

  public:
    enum class Orientation
    {
        Horizontal,
        Vertical
    };

    explicit GSeparator(Orientation orientation, QWidget* parent = nullptr);
};
