#include "Theme.h"

Theme& Theme::instance() {
    static Theme instance;
    return instance;
}

Theme::Theme(QObject* parent) : QObject(parent) {
    m_lightColors = {
        .background = QColor("#ffffff"),
        .surface = QColor("#f8f9fa"),
        .border = QColor("#e9ecef"),
        .primary = QColor("#007bff"),
        .primaryHover = QColor("#0056b3"),
        .text = QColor("#212529"),
        .textSecondary = QColor("#6c757d"),
        .textMuted = QColor("#adb5bd"),
        .success = QColor("#28a745"),
        .warning = QColor("#ffc107"),
        .danger = QColor("#dc3545"),
        .info = QColor("#17a2b8")
    };

    m_darkColors = {
        .background = QColor("#1e1e1e"),
        .surface = QColor("#2d2d30"),
        .border = QColor("#3e3e42"),
        .primary = QColor("#0d7377"),
        .primaryHover = QColor("#14a085"),
        .text = QColor("#ffffff"),
        .textSecondary = QColor("#cccccc"),
        .textMuted = QColor("#969696"),
        .success = QColor("#4caf50"),
        .warning = QColor("#ff9800"),
        .danger = QColor("#f44336"),
        .info = QColor("#2196f3")
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