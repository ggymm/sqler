#include "Theme.h"

Theme& Theme::instance() {
    static Theme instance;
    return instance;
}

Theme::Theme(QObject* parent) : QObject(parent) {
    m_lightColors = {
        .background = QColor(0xffffff),
        .surface = QColor(0xf8f9fa),
        .border = QColor(0xe9ecef),
        .primary = QColor(0x007bff),
        .primaryHover = QColor(0x0056b3),
        .text = QColor(0x212529),
        .textSecondary = QColor(0x6c757d),
        .textMuted = QColor(0xadb5bd),
        .success = QColor(0x28a745),
        .warning = QColor(0xffc107),
        .danger = QColor(0xdc3545),
        .info = QColor(0x17a2b8)
    };

    m_darkColors = {
        .background = QColor(0x1e1e1e),
        .surface = QColor(0x2d2d30),
        .border = QColor(0x3e3e42),
        .primary = QColor(0x0d7377),
        .primaryHover = QColor(0x14a085),
        .text = QColor(0xffffff),
        .textSecondary = QColor(0xcccccc),
        .textMuted = QColor(0x969696),
        .success = QColor(0x4caf50),
        .warning = QColor(0xff9800),
        .danger = QColor(0xf44336),
        .info = QColor(0x2196f3)
    };

    updateColors();
}

void Theme::setDarkMode(bool dark) {
    if (m_isDarkMode != dark) {
        m_isDarkMode = dark;
        updateColors();
        emit themeChanged();
    }
}

void Theme::updateColors() {
    m_colors = m_isDarkMode ? m_darkColors : m_lightColors;
}

QString Theme::getButtonStyle(const QString& variant) const {
    const auto& colors = m_colors;

    if (variant == QStringLiteral("primary")) {
        return QString(
            QStringLiteral("QPushButton {"
            "    background-color: %1;"
            "    color: white;"
            "    border: none;"
            "    border-radius: %2px;"
            "    padding: %3px %4px;"
            "    font-size: %5px;"
            "    font-weight: %6;"
            "    min-width: %7px;"
            "}"
            "QPushButton:hover {"
            "    background-color: %8;"
            "}"
            "QPushButton:pressed {"
            "    background-color: %9;"
            "}")
        ).arg(colors.primary.name())
         .arg(Sizes::borderRadius)
         .arg(Spacing::sm)
         .arg(Spacing::md)
         .arg(Typography::buttonTextSize)
         .arg(Typography::bodyWeight)
         .arg(Sizes::formButtonWidth)
         .arg(colors.primaryHover.name(), colors.primaryHover.darker(110).name());
    }

    if (variant == QStringLiteral("secondary")) {
        return QString(
            "QPushButton {"
            "    background-color: transparent;"
            "    color: %1;"
            "    border: 1px solid %2;"
            "    border-radius: %3px;"
            "    padding: %4px %5px;"
            "    font-size: %6px;"
            "    font-weight: %7;"
            "    min-width: %8px;"
            "}"
            "QPushButton:hover {"
            "    background-color: %9;"
            "}"
            "QPushButton:pressed {"
            "    background-color: %10;"
            "}"
        ).arg(colors.text.name(), colors.border.name())
         .arg(Sizes::borderRadius)
         .arg(Spacing::sm)
         .arg(Spacing::md)
         .arg(Typography::buttonTextSize)
         .arg(Typography::bodyWeight)
         .arg(Sizes::formButtonWidth)
         .arg(colors.surface.name(), colors.border.name());
    }

    if (variant == "dialog") {
        return QString(
            "QPushButton {"
            "    background-color: %1;"
            "    border: 1px solid %2;"
            "    border-radius: %3px;"
            "    text-align: left;"
            "    padding: 0px;"
            "    margin-bottom: %4px;"
            "    min-height: %5px;"
            "}"
            "QPushButton:hover {"
            "    background-color: %6;"
            "    border-color: %7;"
            "}"
            "QPushButton:pressed {"
            "    background-color: %8;"
            "    border-color: %7;"
            "}"
        ).arg(colors.background.name(), colors.border.name())
         .arg(Sizes::borderRadius)
         .arg(Spacing::xs)
         .arg(Sizes::dialogButtonHeight)
         .arg(colors.surface.name(), colors.primary.name(), colors.surface.darker(110).name());
    }

    if (variant == "toolbar") {
        return QString(
            "QPushButton {"
            "    color: %1;"
            "    background-color: transparent;"
            "    border: none;"
            "    padding: 4px 8px;"
            "    border-radius: %2px;"
            "    font-size: 14px;"
            "}"
            "QPushButton:hover {"
            "    background-color: %3;"
            "}"
        ).arg(colors.text.name())
         .arg(Sizes::borderRadius)
         .arg(colors.border.name());
    }

    return getButtonStyle("primary"); // Default fallback
}

QString Theme::getInputStyle() const {
    const auto& colors = m_colors;

    return QString(
        "QLineEdit {"
        "    background-color: %1;"
        "    border: 1px solid %2;"
        "    border-radius: %3px;"
        "    padding: %4px %5px;"
        "    font-size: %6px;"
        "    min-height: %7px;"
        "    color: %8;"
        "}"
        "QLineEdit:focus {"
        "    border-color: %9;"
        "    outline: none;"
        "}"
        "QLineEdit::placeholder {"
        "    color: %10;"
        "}"
        "QSpinBox {"
        "    background-color: %1;"
        "    border: 1px solid %2;"
        "    border-radius: %3px;"
        "    padding: %4px %5px;"
        "    font-size: %6px;"
        "    min-height: %7px;"
        "    color: %8;"
        "    selection-background-color: %9;"
        "}"
        "QSpinBox:focus {"
        "    border-color: %9;"
        "    outline: none;"
        "}"
        "QSpinBox::up-button {"
        "    subcontrol-origin: border;"
        "    subcontrol-position: top right;"
        "    width: 16px;"
        "    height: 50%;"
        "    border-left: 1px solid %2;"
        "    border-bottom: none;"
        "    border-top-right-radius: %3px;"
        "    background-color: %11;"
        "    margin: 1px;"
        "}"
        "QSpinBox::up-button:hover {"
        "    background-color: %9;"
        "}"
        "QSpinBox::up-button:pressed {"
        "    background-color: %2;"
        "}"
        "QSpinBox::down-button {"
        "    subcontrol-origin: border;"
        "    subcontrol-position: bottom right;"
        "    width: 16px;"
        "    height: 50%;"
        "    border-left: 1px solid %2;"
        "    border-top: 1px solid %2;"
        "    border-bottom-right-radius: %3px;"
        "    background-color: %11;"
        "    margin: 1px;"
        "}"
        "QSpinBox::down-button:hover {"
        "    background-color: %9;"
        "}"
        "QSpinBox::down-button:pressed {"
        "    background-color: %2;"
        "}"
        "QSpinBox::up-arrow {"
        "    image: none;"
        "    border-left: 3px solid transparent;"
        "    border-right: 3px solid transparent;"
        "    border-bottom: 4px solid %8;"
        "    width: 0px;"
        "    height: 0px;"
        "    margin-top: 2px;"
        "}"
        "QSpinBox::down-arrow {"
        "    image: none;"
        "    border-left: 3px solid transparent;"
        "    border-right: 3px solid transparent;"
        "    border-top: 4px solid %8;"
        "    width: 0px;"
        "    height: 0px;"
        "    margin-bottom: 2px;"
        "}"
        "QLabel {"
        "    color: %8;"
        "    font-size: %6px;"
        "    font-weight: %12;"
        "}"
    ).arg(colors.surface.name(), colors.border.name())
     .arg(Sizes::borderRadius)
     .arg(Spacing::sm)
     .arg(Spacing::md)
     .arg(Typography::bodySize)
     .arg(Sizes::inputHeight)
     .arg(colors.text.name(), colors.primary.name(), colors.textMuted.name(), colors.surface.lighter(110).name())
     .arg(Typography::bodyWeight);
}

QString Theme::getDialogStyle() const {
    const auto& colors = m_colors;

    return QString(
        "QDialog {"
        "    background-color: %1;"
        "    color: %2;"
        "}"
        "QLabel[class=\"title\"] {"
        "    color: %2;"
        "    font-size: %3px;"
        "    font-weight: %4;"
        "    margin-bottom: %5px;"
        "}"
        "QLabel[class=\"subtitle\"] {"
        "    color: %2;"
        "    font-size: %6px;"
        "    font-weight: %7;"
        "}"
        "QLabel[class=\"description\"] {"
        "    color: %8;"
        "    font-size: %9px;"
        "    margin-top: 2px;"
        "}"
    ).arg(colors.background.name(), colors.text.name())
     .arg(Typography::titleSize)
     .arg(Typography::titleWeight)
     .arg(Spacing::sm)
     .arg(Typography::subtitleSize)
     .arg(Typography::subtitleWeight)
     .arg(colors.textSecondary.name())
     .arg(Typography::bodySize);
}

QString Theme::getScrollAreaStyle() const {
    const auto& colors = m_colors;

    return QString(
        "QScrollArea {"
        "    background-color: %1;"
        "    border: none;"
        "}"
        "QWidget#scrollContent {"
        "    background-color: %1;"
        "}"
        "QScrollBar:vertical {"
        "    border: none;"
        "    background: %2;"
        "    width: 8px;"
        "    border-radius: 4px;"
        "}"
        "QScrollBar::handle:vertical {"
        "    background: %3;"
        "    border-radius: 4px;"
        "    min-height: 20px;"
        "}"
        "QScrollBar::handle:vertical:hover {"
        "    background: %4;"
        "}"
        "QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {"
        "    height: 0px;"
        "}"
    ).arg(colors.background.name(), colors.surface.name(), colors.border.name(), colors.textMuted.name());
}
