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
    // Light palette - 基于Naive UI设计系统重新设计
    m_light = {
        QColor(0xfafafa), // background - 主背景
        QColor(0xffffff), // surface - 卡片/面板背景，使用纯白
        QColor(0xe0e0e6), // border - 边框色，参考Naive UI
        QColor(0x18a058), // primary - Naive UI绿色主色调
        QColor(0x36ad6a), // primaryHover - 悬停状态
        QColor(0x1f2225), // text - 主文本色
        QColor(0x333639), // textSecondary - 次要文本色
        QColor(0x767c82), // textMuted - 弱化文本色
        QColor(0x18a058)  // success - 成功色
    };

    // Dark palette - 使用 Naive UI 暗色主题颜色
    m_dark = {
        QColor(16, 16, 20),         // background - Naive UI neutralBody
        QColor(24, 24, 28),         // surface - Naive UI neutralCard
        QColor(255, 255, 255, 61),  // border - rgba(255, 255, 255, 0.24)
        QColor(0x63e2b7),           // primary - Naive UI primaryDefault
        QColor(0x7fe7c4),           // primaryHover - Naive UI primaryHover
        QColor(255, 255, 255, 230), // text - rgba(255, 255, 255, 0.9)
        QColor(255, 255, 255, 209), // textSecondary - rgba(255, 255, 255, 0.82)
        QColor(255, 255, 255, 133), // textMuted - rgba(255, 255, 255, 0.52)
        QColor(0x63e2b7)            // success - Naive UI successDefault
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
    return buildBaseStyles() + buildLabelStyles() + buildButtonStyles() + buildInputStyles() + buildScrollAreaStyles() + buildListStyles() +
           buildConnectionStyles() + buildSeparatorStyles();
}

QString GTheme::buildBaseStyles() const
{
    const auto& c = m_palette;
    const bool isLight = (m_mode == Mode::Light);
    const QString headerFooterBg = isLight ? NaiveUI::LightColors::hoverColor : NaiveUI::DarkColors::neutralBody;

    return QString(
               "QMainWindow { background-color: %1; color: %2; }"
               "GDialog { background-color: %3; color: %2; }"
               // Dialog pages
               "QWidget#connectionFormPage { background-color: %3; }"
               "QWidget#dbTypePage { background-color: %3; }"
               // Dialog header/footer should be deeper than content
               "QWidget#dialogHeader, QWidget#dialogFooter { background-color: %4; }"
               "QWidget#dialogHeader { border-bottom: 1px solid %5; }"
               "QWidget#dialogFooter { border-top: 1px solid %5; }")
        .arg(c.background.name(), c.text.name(), c.surface.name(), headerFooterBg, c.border.name());
}

QString GTheme::buildLabelStyles() const
{
    const auto& c = m_palette;
    return QString("QLabel { background-color: transparent; color: %1; }"
                   "GLabel { color: %1; font-size: 14px; background-color: transparent; }"
                   "GLabel[gRole=\"title\"] { font-size: 20px; font-weight: 700; color: %1; background-color: transparent; }"
                   "GLabel[gRole=\"subtitle\"] { font-size: 14px; font-weight: 500; color: %2; background-color: transparent; }"
                   "GLabel[gRole=\"caption\"] { font-size: 12px; color: %2; background-color: transparent; }"
                   "GLabel[gRole=\"emphasis\"] { font-size: 14px; font-weight: 600; color: %1; background-color: transparent; }"
                   "GLabel[gRole=\"body\"] { font-size: 14px; color: %1; background-color: transparent; }"
                   "QFormLayout QLabel { background-color: transparent; color: %1; font-weight: 500; }"
                   // Dialog header: make title look secondary to contrast with content
                   "QWidget#dialogHeader GLabel { color: %2; }")
        .arg(c.text.name(), c.textSecondary.name());
}

QString GTheme::buildButtonStyles() const
{
    using namespace GStyle;
    const auto& c = m_palette;

    // 根据当前主题模式选择 Naive UI 颜色
    const bool isLight = (m_mode == Mode::Light);
    const QString dialogHoverColor = isLight ? NaiveUI::LightColors::hoverColor : NaiveUI::DarkColors::hoverColor;
    const QString dialogPressedColor = isLight ? NaiveUI::LightColors::pressedColor : NaiveUI::DarkColors::pressedColor;
    const QString primaryColor = isLight ? NaiveUI::LightColors::primaryDefault : NaiveUI::DarkColors::primaryDefault;
    const QString primaryHoverColor = isLight ? NaiveUI::LightColors::primaryHover : NaiveUI::DarkColors::primaryHover;
    const QString borderColor = isLight ? NaiveUI::LightColors::borderColor : NaiveUI::DarkColors::borderColor;

    return QString("GPushButton { border-radius: %1px; }"
                   "GPushButton[gVariant=\"primary\"] { background-color: %2; color: white; border: none; padding: %3px %4px; min-width: %5px; }"
                   "GPushButton[gVariant=\"primary\"]:hover { background-color: %6; }"
                   "GPushButton[gVariant=\"secondary\"] { background-color: transparent; color: %7; border: 1px solid %8; padding: %3px %4px; "
                   "min-width: %5px; }"
                   "GPushButton[gVariant=\"secondary\"]:hover { background-color: %9; }"
                   "GPushButton[gVariant=\"dialog\"] { background-color: %9; border: 1px solid %8; text-align: left; padding: 0px; margin-bottom: "
                   "%10px; min-height: %11px; }"
                   "GPushButton[gVariant=\"dialog\"]:hover { background-color: %12; border-color: %2; }"
                   "GPushButton[gVariant=\"dialog\"]:pressed { background-color: %13; border-color: %2; }"
                   "GPushButton[gVariant=\"toolbar\"] { color: %7; background-color: transparent; border: none; padding: 4px 8px; }"
                   "GPushButton[gVariant=\"toolbar\"]:hover { background-color: %8; }")
        .arg(QString::number(Sizes::borderRadius))       // 1
        .arg(primaryColor)                               // 2 - Naive UI primary
        .arg(QString::number(Spacing::sm))               // 3
        .arg(QString::number(Spacing::md))               // 4
        .arg(QString::number(Sizes::formButtonWidth))    // 5
        .arg(primaryHoverColor)                          // 6 - Naive UI primary hover
        .arg(c.text.name())                              // 7
        .arg(borderColor)                                // 8 - Naive UI border
        .arg(c.surface.name())                           // 9
        .arg(QString::number(Spacing::xs))               // 10
        .arg(QString::number(Sizes::dialogButtonHeight)) // 11
        .arg(dialogHoverColor)                           // 12 - Naive UI dialog hover
        .arg(dialogPressedColor);                        // 13 - Naive UI dialog pressed
}

QString GTheme::buildInputStyles() const
{
    const auto& c = m_palette;

    // 根据当前主题模式选择 Naive UI 颜色
    const bool isLight = (m_mode == Mode::Light);
    const QString borderColor = isLight ? NaiveUI::LightColors::borderColor : NaiveUI::DarkColors::borderColor;
    const QString hoverColor = isLight ? NaiveUI::LightColors::primaryHover : NaiveUI::DarkColors::primaryHover;
    const QString focusColor = isLight ? NaiveUI::LightColors::primaryHover : NaiveUI::DarkColors::primaryHover;
    const QString placeholderColor = isLight ? NaiveUI::LightColors::placeholderColor : NaiveUI::DarkColors::placeholderColor;
    const QString selectionBgColor = isLight ? NaiveUI::LightColors::primaryColorAlpha20 : NaiveUI::DarkColors::primaryColorAlpha20;
    const QString buttonHoverColor = isLight ? NaiveUI::LightColors::actionColorAlpha02 : NaiveUI::DarkColors::actionColorAlpha02;
    const QString inputBgColor = isLight ? NaiveUI::LightColors::inputColor : NaiveUI::DarkColors::inputColor;

    return QString("GLineEdit { "
                   "background-color: %1; "
                   "border: 1px solid %2; "
                   "border-radius: 6px; "
                   "padding: 10px 12px; "
                   "font-size: 14px; "
                   "min-height: 20px; "
                   "color: %3; "
                   "}"
                   "GLineEdit:hover { "
                   "border-color: %4; "
                   "}"
                   "GLineEdit:focus { "
                   "border-color: %5; "
                   "outline: none; "
                   "}"
                   "GLineEdit::placeholder { "
                   "color: %6; "
                   "}"
                   "GSpinBox { "
                   "background-color: %1; "
                   "border: 1px solid %2; "
                   "border-radius: 6px; "
                   "padding: 10px 12px; "
                   "font-size: 14px; "
                   "min-height: 20px; "
                   "color: %3; "
                   "selection-background-color: %7; "
                   "}"
                   "GSpinBox:hover { "
                   "border-color: %4; "
                   "}"
                   "GSpinBox:focus { "
                   "border-color: %5; "
                   "outline: none; "
                   "}"
                   "GSpinBox::up-button { "
                   "subcontrol-origin: border; "
                   "subcontrol-position: top right; "
                   "width: 20px; "
                   "height: 50%; "
                   "border-left: 1px solid %2; "
                   "border-top-right-radius: 4px; "
                   "background-color: %1; "
                   "margin: 1px; "
                   "}"
                   "GSpinBox::up-button:hover { "
                   "background-color: %8; "
                   "}"
                   "GSpinBox::down-button { "
                   "subcontrol-origin: border; "
                   "subcontrol-position: bottom right; "
                   "width: 20px; "
                   "height: 50%; "
                   "border-left: 1px solid %2; "
                   "border-bottom-right-radius: 4px; "
                   "background-color: %1; "
                   "margin: 1px; "
                   "}"
                   "GSpinBox::down-button:hover { "
                   "background-color: %8; "
                   "}"
                   "GSpinBox::up-arrow { "
                   "image: none; "
                   "border-left: 3px solid transparent; "
                   "border-right: 3px solid transparent; "
                   "border-bottom: 4px solid %3; "
                   "}"
                   "GSpinBox::down-arrow { "
                   "image: none; "
                   "border-left: 3px solid transparent; "
                   "border-right: 3px solid transparent; "
                   "border-top: 4px solid %3; "
                   "}")
        .arg(inputBgColor)      // 1 - background (Naive UI theme-aware)
        .arg(borderColor)       // 2 - default border (Naive UI theme-aware)
        .arg(c.text.name())     // 3 - text color
        .arg(hoverColor)        // 4 - hover border (Naive UI theme-aware)
        .arg(focusColor)        // 5 - focus border (Naive UI theme-aware)
        .arg(placeholderColor)  // 6 - placeholder (Naive UI theme-aware)
        .arg(selectionBgColor)  // 7 - selection background (Naive UI theme-aware)
        .arg(buttonHoverColor); // 8 - button hover (Naive UI theme-aware)
}

QString GTheme::buildScrollAreaStyles() const
{
    const auto& c = m_palette;
    return QString("GScrollArea { background-color: %1; border: none; }"
                   "QScrollArea, QScrollArea > QWidget, QScrollArea > QWidget > QWidget { background-color: %1; }"
                   "QWidget#scrollContent { background-color: %1; }"
                   "QStackedWidget, QStackedWidget > QWidget { background-color: %2; }"
                   "GDialog QWidget { background-color: %2; }"
                   "QScrollBar:vertical { border: none; background: %2; width: 8px; border-radius: 4px; }"
                   "QScrollBar::handle:vertical { background: %3; border-radius: 4px; min-height: 20px; }"
                   "QScrollBar::handle:vertical:hover { background: %4; }"
                   "QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical { height: 0px; }")
        .arg(c.surface.name())
        .arg(c.surface.name())
        .arg(c.border.name())
        .arg(c.textSecondary.name());
}

QString GTheme::buildListStyles() const
{
    return QString("GListWidget { background-color: transparent; border: none; outline: none; }"
                   "GListWidget::item { background-color: transparent; border: none; }"
                   "GListWidget::item:selected { background-color: transparent; }"
                   "GListWidget::item:hover { background-color: transparent; }"
                   "GListWidget::viewport { background-color: transparent; }"
                   "QAbstractItemView { background-color: transparent; }"
                   "QAbstractItemView::viewport { background-color: transparent; }");
}

QString GTheme::buildConnectionStyles() const
{
    using namespace GStyle;
    const auto& c = m_palette;
    return QString("GConnectionItem { background-color: %1; border-radius: %2px; }"
                   "GConnectionItem:hover { background-color: %3; }"
                   "GConnectionItem[gSelected=\"true\"] { background-color: #dbeafe; }"
                   "QLabel[gStatus=\"connected\"] { color: #10b981; }"
                   "QLabel[gStatus=\"disconnected\"] { color: %4; }"
                   "ConnectionPanel { background-color: %1; border-right: 1px solid %5; }"
                   "QWidget#connectionPanel { background-color: %1; border-right: 1px solid %5; }")
        .arg(c.surface.name())
        .arg(QString::number(Sizes::borderRadius))
        .arg(c.textMuted.name())
        .arg(c.textSecondary.name())
        .arg(c.border.name());
}

QString GTheme::buildSeparatorStyles() const
{
    const auto& c = m_palette;
    return QString("GSeparator { background-color: %1; color: %1; }").arg(c.border.name());
}
