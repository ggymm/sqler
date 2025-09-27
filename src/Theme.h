#pragma once

#include <QColor>
#include <QObject>

class Theme : public QObject {
    Q_OBJECT

public:
    static Theme& instance();

    [[nodiscard]] bool isDarkMode() const { return m_isDarkMode; }
    void setDarkMode(bool dark);

    struct Colors {
        QColor background;
        QColor surface;
        QColor border;
        QColor primary;
        QColor primaryHover;
        QColor text;
        QColor textSecondary;
        QColor textMuted;
        QColor success;
        QColor warning;
        QColor danger;
        QColor info;
    };

    struct Spacing {
        static constexpr int xs  = 4;
        static constexpr int sm  = 8;
        static constexpr int md  = 16;
        static constexpr int lg  = 24;
        static constexpr int xl  = 32;
        static constexpr int xxl = 48;
    };

    struct Sizes {
        static constexpr int topBarHeight     = 48;
        static constexpr int sideBarWidth     = 280;
        static constexpr int iconSize         = 20;
        static constexpr int buttonHeight     = 36;
        static constexpr int inputHeight      = 32;
        static constexpr int borderRadius     = 6;
        static constexpr int dialogButtonHeight = 70;
        static constexpr int formButtonWidth = 80;
    };

    struct Typography {
        static constexpr int titleSize         = 18;
        static constexpr int subtitleSize      = 16;
        static constexpr int bodySize          = 14;
        static constexpr int captionSize       = 12;
        static constexpr int buttonTextSize    = 14;
        static constexpr int titleWeight       = 700;
        static constexpr int subtitleWeight    = 600;
        static constexpr int bodyWeight        = 400;
    };

    [[nodiscard]] const Colors& colors() const { return m_colors; }

    // Centralized style generators
    [[nodiscard]] QString getButtonStyle(const QString& variant = "primary") const;
    [[nodiscard]] QString getInputStyle() const;
    [[nodiscard]] QString getDialogStyle() const;
    [[nodiscard]] QString getScrollAreaStyle() const;

signals:
    void themeChanged();

private:
    explicit Theme(QObject* parent = nullptr);
    void updateColors();

    bool   m_isDarkMode = false;
    Colors m_colors;
    Colors m_lightColors;
    Colors m_darkColors;
};