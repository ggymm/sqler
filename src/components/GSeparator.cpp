#include "GSeparator.h"

GSeparator::GSeparator(Orientation orientation, QWidget* parent) : QFrame(parent) {
    setFrameShape(orientation == Orientation::Horizontal ? QFrame::HLine : QFrame::VLine);
    setLineWidth(1);
    if (orientation == Orientation::Horizontal) {
        setFixedHeight(1);
    } else {
        setFixedWidth(1);
    }
}
