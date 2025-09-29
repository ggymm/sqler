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
    return buildBaseStyles() +
           buildLabelStyles() +
           buildButtonStyles() +
           buildInputStyles() +
           buildScrollAreaStyles() +
           buildListStyles() +
           buildConnectionStyles() +
           buildSeparatorStyles();
}

QString GTheme::buildBaseStyles() const
{
    const auto& c = m_palette;
    return QString(
        "QMainWindow { background-color: %1; color: %2; }"
        "GDialog { background-color: %3; color: %2; }"
        "QWidget#connectionFormPage { background-color: %1; }"
        "QWidget#dbTypePage { background-color: %3; }"
    ).arg(c.background.name(), c.text.name(), c.surface.name());
}

QString GTheme::buildLabelStyles() const
{
    const auto& c = m_palette;
    return QString(
        "QLabel { background-color: transparent; color: %1; }"
        "GLabel { color: %1; font-size: 14px; background-color: transparent; }"
        "GLabel[gRole=\"title\"] { font-size: 20px; font-weight: 700; color: %1; background-color: transparent; }"
        "GLabel[gRole=\"subtitle\"] { font-size: 14px; font-weight: 500; color: %2; background-color: transparent; }"
        "GLabel[gRole=\"caption\"] { font-size: 12px; color: %2; background-color: transparent; }"
        "GLabel[gRole=\"emphasis\"] { font-size: 14px; font-weight: 600; color: %1; background-color: transparent; }"
        "GLabel[gRole=\"body\"] { font-size: 14px; color: %1; background-color: transparent; }"
        "QFormLayout QLabel { background-color: transparent; color: %1; font-weight: 500; }"
    ).arg(c.text.name(), c.textSecondary.name());
}

QString GTheme::buildButtonStyles() const
{
    using namespace GStyle;
    const auto& c = m_palette;
    return QString(
        "GPushButton { border-radius: %1px; }"
        "GPushButton[gVariant=\"primary\"] { background-color: %2; color: white; border: none; padding: %3px %4px; min-width: %5px; }"
        "GPushButton[gVariant=\"primary\"]:hover { background-color: %6; }"
        "GPushButton[gVariant=\"secondary\"] { background-color: transparent; color: %7; border: 1px solid %8; padding: %3px %4px; min-width: %5px; }"
        "GPushButton[gVariant=\"secondary\"]:hover { background-color: %9; }"
        "GPushButton[gVariant=\"dialog\"] { background-color: %9; border: 1px solid %8; text-align: left; padding: 0px; margin-bottom: %10px; min-height: %11px; }"
        "GPushButton[gVariant=\"dialog\"]:hover { background-color: %12; border-color: %13; }"
        "GPushButton[gVariant=\"dialog\"]:pressed { background-color: #f5f5f5; border-color: %13; }"
        "GPushButton[gVariant=\"toolbar\"] { color: %7; background-color: transparent; border: none; padding: 4px 8px; }"
        "GPushButton[gVariant=\"toolbar\"]:hover { background-color: %8; }"
    ).arg(Sizes::borderRadius)      // 1
     .arg(c.primary.name())         // 2
     .arg(Spacing::sm)              // 3
     .arg(Spacing::md)              // 4
     .arg(Sizes::formButtonWidth)   // 5
     .arg(c.primaryHover.name())    // 6
     .arg(c.text.name())            // 7
     .arg(c.border.name())          // 8
     .arg(c.surface.name())         // 9
     .arg(Spacing::xs)              // 10
     .arg(Sizes::dialogButtonHeight) // 11
     .arg(c.textMuted.name())       // 12
     .arg(c.textSecondary.name());  // 13
}

QString GTheme::buildInputStyles() const
{
    const auto& c = m_palette;
    return QString(
        "GLineEdit { background-color: white; border: 1px solid %1; border-radius: 6px; padding: 12px 16px; font-size: 14px; min-height: 20px; color: %2; }"
        "GLineEdit:focus { border-color: %3; }"
        "GLineEdit:hover { border-color: %4; }"
        "GLineEdit::placeholder { color: #9ca3af; }"
        "GSpinBox { background-color: white; border: 1px solid %1; border-radius: 6px; padding: 12px 16px; font-size: 14px; min-height: 20px; color: %2; selection-background-color: %3; }"
        "GSpinBox:focus { border-color: %3; }"
        "GSpinBox:hover { border-color: %4; }"
        "GSpinBox::up-button { subcontrol-origin: border; subcontrol-position: top right; width: 16px; height: 50%; border-left: 1px solid %1; border-top-right-radius: 6px; background-color: white; margin: 1px; }"
        "GSpinBox::up-button:hover { background-color: %3; color: white; }"
        "GSpinBox::down-button { subcontrol-origin: border; subcontrol-position: bottom right; width: 16px; height: 50%; border-left: 1px solid %1; border-bottom-right-radius: 6px; background-color: white; margin: 1px; }"
        "GSpinBox::down-button:hover { background-color: %3; color: white; }"
        "GSpinBox::up-arrow { image: none; border-left: 3px solid transparent; border-right: 3px solid transparent; border-bottom: 4px solid %2; }"
        "GSpinBox::down-arrow { image: none; border-left: 3px solid transparent; border-right: 3px solid transparent; border-top: 4px solid %2; }"
    ).arg(c.border.name())
     .arg(c.text.name())
     .arg(c.primary.name())
     .arg(c.textSecondary.name());
}

QString GTheme::buildScrollAreaStyles() const
{
    const auto& c = m_palette;
    return QString(
        "GScrollArea { background-color: %1; border: none; }"
        "QScrollArea, QScrollArea > QWidget, QScrollArea > QWidget > QWidget { background-color: %1; }"
        "QWidget#scrollContent { background-color: %1; }"
        "QStackedWidget, QStackedWidget > QWidget { background-color: %2; }"
        "GDialog QWidget { background-color: %2; }"
        "QScrollBar:vertical { border: none; background: %2; width: 8px; border-radius: 4px; }"
        "QScrollBar::handle:vertical { background: %3; border-radius: 4px; min-height: 20px; }"
        "QScrollBar::handle:vertical:hover { background: %4; }"
        "QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical { height: 0px; }"
    ).arg(c.background.name())
     .arg(c.surface.name())
     .arg(c.border.name())
     .arg(c.textSecondary.name());
}

QString GTheme::buildListStyles() const
{
    return QString(
        "GListWidget { background-color: transparent; border: none; outline: none; }"
        "GListWidget::item { background-color: transparent; border: none; }"
        "GListWidget::item:selected { background-color: transparent; }"
        "GListWidget::item:hover { background-color: transparent; }"
        "GListWidget::viewport { background-color: transparent; }"
        "QAbstractItemView { background-color: transparent; }"
        "QAbstractItemView::viewport { background-color: transparent; }"
    );
}

QString GTheme::buildConnectionStyles() const
{
    using namespace GStyle;
    const auto& c = m_palette;
    return QString(
        "GConnectionItem { background-color: %1; border-radius: %2px; }"
        "GConnectionItem:hover { background-color: %3; }"
        "GConnectionItem[gSelected=\"true\"] { background-color: #dbeafe; }"
        "QLabel[gStatus=\"connected\"] { color: #10b981; }"
        "QLabel[gStatus=\"disconnected\"] { color: %4; }"
        "ConnectionPanel { background-color: %1; border-right: 1px solid %5; }"
        "QWidget#connectionPanel { background-color: %1; border-right: 1px solid %5; }"
    ).arg(c.surface.name())
     .arg(Sizes::borderRadius)
     .arg(c.textMuted.name())
     .arg(c.textSecondary.name())
     .arg(c.border.name());
}

QString GTheme::buildSeparatorStyles() const
{
    const auto& c = m_palette;
    return QString(
        "GSeparator { background-color: %1; color: %1; }"
    ).arg(c.border.name());
}
