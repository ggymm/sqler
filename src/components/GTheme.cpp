#include "GTheme.h"

#include "GStyle.h"

#include <QApplication>
#include <QDebug>

GTheme& GTheme::instance()
{
    static GTheme inst;
    return inst;
}

GTheme::GTheme(QObject* parent) : QObject(parent)
{
    // Light palette - 基于2024现代设计系统重新设计
    m_light = {
        QColor(0xfafafa), // background - off-white主背景，避免眼部疲劳
        QColor(0xf5f5f5), // surface - 面板背景，创建层次感
        QColor(0xd1d5db), // border - 柔和边框，基于Tailwind gray-300
        QColor(0x3b82f6), // primary - 现代蓝色，基于Tailwind blue-500
        QColor(0x2563eb), // primaryHover - 深蓝色悬停，基于Tailwind blue-600
        QColor(0x111827), // text - 深灰色文本，基于Tailwind gray-900
        QColor(0x6b7280), // textSecondary - 中等灰色，基于Tailwind gray-500
        QColor(0x9ca3af), // textMuted - 浅灰色，基于Tailwind gray-400
        QColor(0x10b981)  // success - 现代绿色，基于Tailwind emerald-500
    };

    // Dark palette
    m_dark = {
        QColor(0x1e1e1e), // background
        QColor(0x4a4a4a), // surface - 更亮的灰色
        QColor(0x5a5a5a), // border - 更亮的边框
        QColor(0x0d7377), // primary
        QColor(0x14a085), // primaryHover
        QColor(0xffffff), // text
        QColor(0xcccccc), // textSecondary
        QColor(0x969696), // textMuted
        QColor(0x4caf50)  // success
    };

    updatePalette();
}

void GTheme::setMode(Mode mode)
{
    if (m_mode != mode)
    {
        m_mode = mode;
        updatePalette();
        applyToApp();
        emit themeChanged();
    }
}

void GTheme::updatePalette()
{
    m_palette = (m_mode == Mode::Light) ? m_light : m_dark;
}

void GTheme::applyToApp()
{
    qApp->setStyleSheet(buildGlobalStyleSheet());
}

QString GTheme::buildGlobalStyleSheet() const
{
    using namespace GStyle;
    const auto& c = m_palette;

    const QString base =
        QString(
            // Base surfaces
            "QMainWindow { background-color: %1; color: %2; }"
            "GDialog { background-color: %3; color: %2; }"
            // Labels - 确保所有标签透明背景
            "QLabel { background-color: transparent; color: %2; }"
            "GLabel { color: %2; font-size: 14px; background-color: transparent; }"
            "GLabel[gRole=\"title\"] { font-size: 20px; font-weight: 700; color: %2; background-color: transparent; }"
            "GLabel[gRole=\"subtitle\"] { font-size: 14px; font-weight: 500; color: %4; background-color: transparent; }"
            "GLabel[gRole=\"caption\"] { font-size: 12px; color: %4; background-color: transparent; }"
            "GLabel[gRole=\"emphasis\"] { font-size: 14px; font-weight: 600; color: %2; background-color: transparent; }"
            "GLabel[gRole=\"body\"] { font-size: 14px; color: %2; background-color: transparent; }"
            "QFormLayout QLabel { background-color: transparent; color: %2; font-weight: 500; }"
            // Buttons
            "GPushButton { border-radius: %5px; }"
            "GPushButton[gVariant=\"primary\"] { background-color: %6; color: white; border: none; padding: %7px %8px; min-width: %9px; }"
            "GPushButton[gVariant=\"primary\"]:hover { background-color: %10; }"
            "GPushButton[gVariant=\"secondary\"] { background-color: transparent; color: %2; border: 1px solid %11; padding: %7px %8px; min-width: "
            "%9px; }"
            "GPushButton[gVariant=\"secondary\"]:hover { background-color: %3; }"
            "GPushButton[gVariant=\"dialog\"] { background-color: %3; border: 1px solid %11; text-align: left; padding: 0px; margin-bottom: %12px; "
            "min-height: %13px; }"
            "GPushButton[gVariant=\"dialog\"]:hover { background-color: %14; border-color: %15; }"
            "GPushButton[gVariant=\"dialog\"]:pressed { background-color: %20; border-color: %15; }"
            "GPushButton[gVariant=\"toolbar\"] { color: %2; background-color: transparent; border: none; padding: 4px 8px; }"
            "GPushButton[gVariant=\"toolbar\"]:hover { background-color: %11; }"
            // Inputs - 现代化输入框设计
            "GLineEdit { background-color: white; border: 1px solid %11; border-radius: 6px; padding: 12px 16px; font-size: 14px; min-height: 20px; "
            "color: %2; transition: all 0.2s ease; }"
            "GLineEdit:focus { border-color: %6; outline: none; box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1); }"
            "GLineEdit:hover { border-color: %21; }"
            "GLineEdit::placeholder { color: %21; }"
            "GSpinBox { background-color: white; border: 1px solid %11; border-radius: 6px; padding: 12px 16px; font-size: 14px; min-height: 20px; "
            "color: %2; selection-background-color: %6; transition: all 0.2s ease; }"
            "GSpinBox:focus { border-color: %6; outline: none; box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1); }"
            "GSpinBox:hover { border-color: %21; }"
            "GSpinBox::up-button { subcontrol-origin: border; subcontrol-position: top right; width: 16px; height: 50%; border-left: 1px solid %11; "
            "border-top-right-radius: %5px; background-color: %1; margin: 1px; }"
            "GSpinBox::up-button:hover { background-color: %6; }"
            "GSpinBox::down-button { subcontrol-origin: border; subcontrol-position: bottom right; width: 16px; height: 50%; border-left: 1px solid "
            "%11; border-bottom-right-radius: %5px; background-color: %1; margin: 1px; }"
            "GSpinBox::down-button:hover { background-color: %6; }"
            "GSpinBox::up-arrow { image: none; border-left: 3px solid transparent; border-right: 3px solid transparent; border-bottom: 4px solid %2; "
            "}"
            "GSpinBox::down-arrow { image: none; border-left: 3px solid transparent; border-right: 3px solid transparent; border-top: 4px solid %2; }"
            // Scroll area + scrollbars
            "GScrollArea { background-color: %1; border: none; }"
            "QScrollArea, QScrollArea > QWidget, QScrollArea > QWidget > QWidget { background-color: %1; }"
            "QWidget#scrollContent { background-color: %1; }"
            "QStackedWidget, QStackedWidget > QWidget { background-color: %3; }"
            "GDialog QWidget { background-color: %3; }"
            "QScrollBar:vertical { border: none; background: %3; width: 8px; border-radius: 4px; }"
            "QScrollBar::handle:vertical { background: %11; border-radius: 4px; min-height: 20px; }"
            "QScrollBar::handle:vertical:hover { background: %4; }"
            "QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical { height: 0px; }"
            // List (disable default selection painting; items render themselves)
            "GListWidget { background-color: transparent; border: none; outline: none; }"
            "GListWidget::item { background-color: transparent; border: none; }"
            "GListWidget::item:selected { background-color: transparent; }"
            "GListWidget::item:hover { background-color: transparent; }"
            "GListWidget::viewport { background-color: transparent; }"
            "QAbstractItemView { background-color: transparent; }"
            "QAbstractItemView::viewport { background-color: transparent; }"
            // Connection item + left panel
            "GConnectionItem { background-color: %3; border-radius: %5px; }"
            "GConnectionItem:hover { background-color: %14; }"
            "GConnectionItem[gSelected=\"true\"] { background-color: %18; }"
            "QLabel[gStatus=\"connected\"] { color: %19; }"
            "QLabel[gStatus=\"disconnected\"] { color: %4; }"
            "ConnectionPanel { background-color: %3; border-right: 1px solid %11; }"
            "QWidget#connectionPanel { background-color: %3; border-right: 1px solid %11; }"
            "QWidget#dbTypePage { background-color: %3; }"
            "QWidget#connectionFormPage { background-color: %1; }"
            // Separator
            "GSeparator { background-color: %11; color: %11; }")
            .arg(c.background.name())           // 1
            .arg(c.text.name())                 // 2
            .arg(c.surface.name())              // 3
            .arg(c.textSecondary.name())        // 4
            .arg(Sizes::borderRadius)           // 5
            .arg(c.primary.name())              // 6
            .arg(Spacing::sm)                   // 7
            .arg(Spacing::md)                   // 8
            .arg(Sizes::formButtonWidth)        // 9
            .arg(c.primaryHover.name())         // 10
            .arg(c.border.name())               // 11
            .arg(Spacing::xs)                   // 12
            .arg(Sizes::dialogButtonHeight)     // 13
            .arg(c.surface.darker(108).name())  // 14 (hover: slightly darker)
            .arg(c.border.darker(110).name())   // 15
            .arg(Sizes::inputHeight)            // 16
            .arg(c.surface.lighter(110).name()) // 17
            .arg(c.primary.lighter(180).name()) // 18
            .arg(c.success.name())              // 19
            .arg(c.surface.darker(116).name())  // 20 (pressed: deeper)
            .arg(c.textMuted.name());           // 21 (input hover border)

    return base;
}
