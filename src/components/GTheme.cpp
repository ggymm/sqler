#include "GTheme.h"

#include "GStyle.h"

#include <QApplication>

GTheme& GTheme::instance() {
    static GTheme inst;
    return inst;
}

GTheme::GTheme(QObject* parent) : QObject(parent) {
    m_light = {
        QColor(0xfafafa),
        QColor(0xffffff),
        QColor(0xe0e0e6),
        QColor(0x18a058),
        QColor(0x36ad6a),
        QColor(0x1f2225),
        QColor(0x333639),
        QColor(0x767c82),
        QColor(0x18a058)};

    m_dark = {
        QColor(16, 16, 20),
        QColor(24, 24, 28),
        QColor(255, 255, 255, 61),
        QColor(0x63e2b7),
        QColor(0x7fe7c4),
        QColor(255, 255, 255, 230),
        QColor(255, 255, 255, 209),
        QColor(255, 255, 255, 133),
        QColor(0x63e2b7)};

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

void GTheme::updatePalette() { m_palette = (m_mode == Mode::Light) ? m_light : m_dark; }

void GTheme::applyToApp() { qApp->setStyleSheet(buildGlobalStyleSheet()); }

QString GTheme::buildGlobalStyleSheet() const {
    return buildBaseStyles() + buildLabelStyles() + buildButtonStyles() + buildInputStyles() + buildScrollAreaStyles() + buildListStyles() +
           buildConnectionStyles() + buildSeparatorStyles();
}

QString GTheme::buildBaseStyles() const {
    const auto& c = m_palette;
    const bool isLight = (m_mode == Mode::Light);
    const QString headerFooterBg = isLight ? NaiveUI::LightColors::hoverColor : NaiveUI::DarkColors::neutralBody;

    return QString(
               "QMainWindow { background-color: %1; color: %2; }"
               "GDialog { background-color: %3; color: %2; }"
               "QWidget#connectionFormPage { background-color: %3; }"
               "QWidget#dbTypePage { background-color: %3; }"
               "QWidget#dialogHeader, QWidget#dialogFooter { background-color: %4; }"
               "QWidget#dialogHeader { border-bottom: 1px solid %5; }"
               "QWidget#dialogFooter { border-top: 1px solid %5; }"
               "QWidget#topMenuBar { border-bottom: 1px solid %5; }"
               "TopMenuBar { border-bottom: 1px solid %5; }")
        .arg(c.background.name(), c.text.name(), c.surface.name(), headerFooterBg, c.border.name());
}

QString GTheme::buildLabelStyles() const {
    const auto& c = m_palette;
    return QString(
               "QLabel { background-color: transparent; color: %1; }"
               "GLabel { background-color: transparent; color: %1; font-size: 14px; }"
               "GLabel[gRole=\"title\"] { background-color: transparent; color: %1; font-size: 20px; font-weight: 700; }"
               "GLabel[gRole=\"subtitle\"] { background-color: transparent; color: %2; font-size: 14px; font-weight: 500; }"
               "GLabel[gRole=\"caption\"] { background-color: transparent; color: %2; font-size: 12px; }"
               "GLabel[gRole=\"emphasis\"] { background-color: transparent; color: %1; font-size: 14px; font-weight: 600; }"
               "GLabel[gRole=\"body\"] { background-color: transparent; color: %1; font-size: 14px; }"
               "QFormLayout QLabel { background-color: transparent; color: %1; font-weight: 500; }"

               "QWidget#dialogHeader GLabel { color: %2; }")
        .arg(c.text.name(), c.textSecondary.name());
}

QString GTheme::buildButtonStyles() const {
    using namespace GStyle;
    const auto& c = m_palette;

    const bool isLight = (m_mode == Mode::Light);
    const QString neutralHoverColor = isLight ? NaiveUI::LightColors::hoverColor : NaiveUI::DarkColors::hoverColor;
    const QString neutralPressedColor = isLight ? NaiveUI::LightColors::pressedColor : NaiveUI::DarkColors::pressedColor;
    const QString primaryColor = isLight ? NaiveUI::LightColors::primaryDefault : NaiveUI::DarkColors::primaryDefault;
    const QString primaryHoverColor = isLight ? NaiveUI::LightColors::primaryHover : NaiveUI::DarkColors::primaryHover;
    const QString primaryActiveColor = isLight ? NaiveUI::LightColors::primaryActive : NaiveUI::DarkColors::primaryActive;
    const QString borderColor = isLight ? NaiveUI::LightColors::borderColor : NaiveUI::DarkColors::borderColor;
    const QString toolbarHoverBg = isLight ? NaiveUI::LightColors::buttonColor2Hover : NaiveUI::DarkColors::buttonColor2Hover;
    const QString toolbarPressedBg = isLight ? NaiveUI::LightColors::buttonColor2Pressed : NaiveUI::DarkColors::buttonColor2Pressed;

    return QString(
               "GPushButton { border-radius: %1px; }"
               "GPushButton[gVariant=\"primary\"] { background-color: %2; border: none; padding: %3px %4px; min-width: %5px; color: white; }"
               "GPushButton[gVariant=\"primary\"]:hover { background-color: %6; }"
               "GPushButton[gVariant=\"primary\"]:pressed { background-color: %7; }"

               "GPushButton[gVariant=\"secondary\"] { background-color: transparent; border: 1px solid %8; padding: %3px %4px; min-width: %5px; color: %9; }"
               "GPushButton[gVariant=\"secondary\"]:hover { background-color: %10; }"
               "GPushButton[gVariant=\"secondary\"]:pressed { background-color: %11; }"

               "GPushButton[gVariant=\"dialog\"] { background-color: %12; border: 1px solid %8; padding: 0px; margin-bottom: %13px; min-height: %14px; text-align: left; }"
               "GPushButton[gVariant=\"dialog\"]:hover { background-color: %10; border-color: %2; }"
               "GPushButton[gVariant=\"dialog\"]:pressed { background-color: %11; border-color: %2; }"

               "GPushButton[gVariant=\"toolbar\"] { background-color: transparent; border: none; padding: 4px 8px; color: %9; }"
               "GPushButton[gVariant=\"toolbar\"]:hover { background-color: %15; }"
               "GPushButton[gVariant=\"toolbar\"]:pressed { background-color: %16; }"

               "GPushButton[gVariant=\"neutral\"] { background-color: %12; border: 1px solid %8; padding: %3px %4px; min-width: %5px; color: %9; }"
               "GPushButton[gVariant=\"neutral\"]:hover { background-color: %10; }"
               "GPushButton[gVariant=\"neutral\"]:pressed { background-color: %11; }")
        .arg(QString::number(Sizes::borderRadius))
        .arg(primaryColor)
        .arg(QString::number(Spacing::sm))
        .arg(QString::number(Spacing::md))
        .arg(QString::number(Sizes::formButtonWidth))
        .arg(primaryHoverColor)
        .arg(primaryActiveColor)
        .arg(borderColor)
        .arg(c.text.name())
        .arg(neutralHoverColor)
        .arg(neutralPressedColor)
        .arg(c.surface.name())
        .arg(QString::number(Spacing::xs))
        .arg(QString::number(Sizes::dialogButtonHeight))
        .arg(toolbarHoverBg)
        .arg(toolbarPressedBg);
}

QString GTheme::buildInputStyles() const {
    const auto& c = m_palette;

    const bool isLight = (m_mode == Mode::Light);
    const QString borderColor = isLight ? NaiveUI::LightColors::borderColor : NaiveUI::DarkColors::borderColor;
    const QString hoverColor = isLight ? NaiveUI::LightColors::primaryHover : NaiveUI::DarkColors::primaryHover;
    const QString focusColor = isLight ? NaiveUI::LightColors::primaryHover : NaiveUI::DarkColors::primaryHover;
    const QString placeholderColor = isLight ? NaiveUI::LightColors::placeholderColor : NaiveUI::DarkColors::placeholderColor;
    const QString selectionBgColor = isLight ? NaiveUI::LightColors::primaryColorAlpha20 : NaiveUI::DarkColors::primaryColorAlpha20;
    const QString buttonHoverColor = isLight ? NaiveUI::LightColors::actionColorAlpha02 : NaiveUI::DarkColors::actionColorAlpha02;
    const QString inputBgColor = isLight ? NaiveUI::LightColors::inputColor : NaiveUI::DarkColors::inputColor;

    return QString(
               "GLineEdit { background-color: %1; border: 1px solid %2; border-radius: 6px; padding: 10px 12px; min-height: 20px; color: %3; font-size: 14px; }"
               "GLineEdit:hover { border-color: %4; }"
               "GLineEdit:focus { border-color: %5; outline: none; }"
               "GLineEdit::placeholder { color: %6; }"
               "GSpinBox { background-color: %1; border: 1px solid %2; border-radius: 6px; padding: 10px 12px; min-height: 20px; color: %3; font-size: 14px; selection-background-color: %7; }"
               "GSpinBox:hover { border-color: %4; }"
               "GSpinBox:focus { border-color: %5; outline: none; }"
               "GSpinBox::up-button { background-color: %1; border-left: 1px solid %2; border-top-right-radius: 4px; subcontrol-origin: border; subcontrol-position: top right; width: 20px; height: 50%; margin: 1px; }"
               "GSpinBox::up-button:hover { background-color: %8; }"
               "GSpinBox::down-button { background-color: %1; border-left: 1px solid %2; border-bottom-right-radius: 4px; subcontrol-origin: border; subcontrol-position: bottom right; width: 20px; height: 50%; margin: 1px; }"
               "GSpinBox::down-button:hover { background-color: %8; }"
               "GSpinBox::up-arrow { border-left: 3px solid transparent; border-right: 3px solid transparent; border-bottom: 4px solid %3; image: none; }"
               "GSpinBox::down-arrow { border-left: 3px solid transparent; border-right: 3px solid transparent; border-top: 4px solid %3; image: none; }")
        .arg(inputBgColor, borderColor, c.text.name(), hoverColor, focusColor, placeholderColor, selectionBgColor, buttonHoverColor);
}

QString GTheme::buildScrollAreaStyles() const {
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
               "QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical { height: 0px; }")
        .arg(c.surface.name(), c.surface.name(), c.border.name(), c.textSecondary.name());
}

QString GTheme::buildListStyles() const {
    return QString(
        "GListWidget { background-color: transparent; border: none; outline: none; }"
        "GListWidget::item { background-color: transparent; border: none; }"
        "GListWidget::item:selected { background-color: transparent; }"
        "GListWidget::item:hover { background-color: transparent; }"
        "GListWidget::viewport { background-color: transparent; }"
        "QAbstractItemView { background-color: transparent; }"
        "QAbstractItemView::viewport { background-color: transparent; }");
}

QString GTheme::buildConnectionStyles() const {
    using namespace GStyle;
    const auto& c = m_palette;
    return QString(
               "GConnectionItem { background-color: %1; border-radius: %2px; }"
               "GConnectionItem:hover { background-color: %3; }"
               "GConnectionItem[gSelected=\"true\"] { background-color: #dbeafe; }"
               "QLabel[gStatus=\"connected\"] { color: #10b981; }"
               "QLabel[gStatus=\"disconnected\"] { color: %4; }"
               "ConnectionPanel { background-color: %1; border-right: 1px solid %5; }"
               "QWidget#connectionPanel { background-color: %1; border-right: 1px solid %5; }")
        .arg(c.surface.name(), QString::number(Sizes::borderRadius), c.textMuted.name(), c.textSecondary.name(), c.border.name());
}

QString GTheme::buildSeparatorStyles() const {
    const auto& c = m_palette;
    return QString("GSeparator { background-color: %1; color: %1; }").arg(c.border.name());
}
