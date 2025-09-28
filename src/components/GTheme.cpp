#include "GTheme.h"
#include "GStyle.h"
#include <QApplication>

GTheme& GTheme::instance() {
    static GTheme inst;
    return inst;
}

GTheme::GTheme(QObject* parent) : QObject(parent) {
    // Light palette
    m_light = {
        QColor(0xffffff), // background
        QColor(0xf8f9fa), // surface
        QColor(0xe9ecef), // border
        QColor(0x007bff), // primary
        QColor(0x0056b3), // primaryHover
        QColor(0x212529), // text
        QColor(0x6c757d), // textSecondary
        QColor(0xadb5bd), // textMuted
        QColor(0x28a745)  // success
    };

    // Dark palette
    m_dark = {
        QColor(0x1e1e1e), // background
        QColor(0x2d2d30), // surface
        QColor(0x3e3e42), // border
        QColor(0x0d7377), // primary
        QColor(0x14a085), // primaryHover
        QColor(0xffffff), // text
        QColor(0xcccccc), // textSecondary
        QColor(0x969696), // textMuted
        QColor(0x4caf50)  // success
    };

    updatePalette();
}

void GTheme::setMode(Mode mode) {
    if (m_mode != mode) {
        m_mode = mode;
        updatePalette();
        applyToApp();
        emit themeChanged();
    }
}

void GTheme::updatePalette() {
    m_palette = (m_mode == Mode::Light) ? m_light : m_dark;
}

void GTheme::applyToApp() {
    qApp->setStyleSheet(buildGlobalStyleSheet());
}

QString GTheme::buildGlobalStyleSheet() const {
    using namespace GStyle;
    const auto& c = m_palette;

    const QString base = QString(
        // Base surfaces
        "QMainWindow, QWidget { background-color: %1; color: %2; }"
        "GDialog { background-color: %3; color: %2; }"
        // Labels
        "GLabel { color: %2; font-size: 14px; }"
        "GLabel[gRole=\"title\"] { font-size: 20px; font-weight: 700; color: %2; }"
        "GLabel[gRole=\"subtitle\"] { font-size: 14px; font-weight: 500; color: %4; }"
        "GLabel[gRole=\"caption\"] { font-size: 12px; color: %4; }"
        "GLabel[gRole=\"emphasis\"] { font-size: 14px; font-weight: 600; color: %2; }"
        // Buttons
        "GPushButton { border-radius: %5px; }"
        "GPushButton[gVariant=\"primary\"] { background-color: %6; color: white; border: none; padding: %7px %8px; min-width: %9px; }"
        "GPushButton[gVariant=\"primary\"]:hover { background-color: %10; }"
        "GPushButton[gVariant=\"secondary\"] { background-color: transparent; color: %2; border: 1px solid %11; padding: %7px %8px; min-width: %9px; }"
        "GPushButton[gVariant=\"secondary\"]:hover { background-color: %3; }"
        "GPushButton[gVariant=\"dialog\"] { background-color: %3; border: 1px solid %11; text-align: left; padding: 0px; margin-bottom: %12px; min-height: %13px; }"
        "GPushButton[gVariant=\"dialog\"]:hover { background-color: %14; border-color: %15; }"
        "GPushButton[gVariant=\"toolbar\"] { color: %2; background-color: transparent; border: none; padding: 4px 8px; }"
        "GPushButton[gVariant=\"toolbar\"]:hover { background-color: %11; }"
        // Inputs
        "GLineEdit { background-color: %3; border: 1px solid %11; border-radius: %5px; padding: %7px %8px; font-size: 14px; min-height: %16px; color: %2; }"
        "GLineEdit:focus { border-color: %6; }"
        "GLineEdit::placeholder { color: %4; }"
        "GSpinBox { background-color: %3; border: 1px solid %11; border-radius: %5px; padding: %7px %8px; font-size: 14px; min-height: %16px; color: %2; selection-background-color: %6; }"
        "GSpinBox:focus { border-color: %6; }"
        "GSpinBox::up-button { subcontrol-origin: border; subcontrol-position: top right; width: 16px; height: 50%%; border-left: 1px solid %11; border-top-right-radius: %5px; background-color: %17; margin: 1px; }"
        "GSpinBox::up-button:hover { background-color: %6; }"
        "GSpinBox::down-button { subcontrol-origin: border; subcontrol-position: bottom right; width: 16px; height: 50%%; border-left: 1px solid %11; border-bottom-right-radius: %5px; background-color: %17; margin: 1px; }"
        "GSpinBox::down-button:hover { background-color: %6; }"
        "GSpinBox::up-arrow { image: none; border-left: 3px solid transparent; border-right: 3px solid transparent; border-bottom: 4px solid %2; }"
        "GSpinBox::down-arrow { image: none; border-left: 3px solid transparent; border-right: 3px solid transparent; border-top: 4px solid %2; }"
        // Scroll area + scrollbars
        "GScrollArea { background-color: %1; border: none; }"
        "QWidget#scrollContent { background-color: %1; }"
        "QScrollBar:vertical { border: none; background: %3; width: 8px; border-radius: 4px; }"
        "QScrollBar::handle:vertical { background: %11; border-radius: 4px; min-height: 20px; }"
        "QScrollBar::handle:vertical:hover { background: %4; }"
        "QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical { height: 0px; }"
        // List
        "GListWidget { background-color: transparent; border: none; outline: none; }"
        // Connection item
        "GConnectionItem { background-color: %1; border-radius: %5px; }"
        "GConnectionItem[gSelected=\"true\"] { background-color: %18; }"
        "QLabel[gStatus=\"connected\"] { color: %19; }"
        "QLabel[gStatus=\"disconnected\"] { color: %4; }"
        // Separator
        "GSeparator { background-color: %11; color: %11; }"
    )
    .arg(c.background.name())            // 1
    .arg(c.text.name())                  // 2
    .arg(c.surface.name())               // 3
    .arg(c.textSecondary.name())         // 4
    .arg(Sizes::borderRadius)            // 5
    .arg(c.primary.name())               // 6
    .arg(Spacing::sm)                    // 7
    .arg(Spacing::md)                    // 8
    .arg(Sizes::formButtonWidth)         // 9
    .arg(c.primaryHover.name())          // 10
    .arg(c.border.name())                // 11
    .arg(Spacing::xs)                    // 12
    .arg(Sizes::dialogButtonHeight)      // 13
    .arg(c.surface.lighter(105).name())  // 14
    .arg(c.border.darker(110).name())    // 15
    .arg(Sizes::inputHeight)             // 16
    .arg(c.surface.lighter(110).name())  // 17
    .arg(c.primary.lighter(180).name())  // 18
    .arg(c.success.name());              // 19

    return base;
}
