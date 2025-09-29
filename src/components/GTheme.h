#pragma once

#include <QColor>
#include <QObject>

// Naive UI 颜色系统
namespace NaiveUI {
    // 亮色主题颜色 (来自 naive-ui/src/_styles/common/light.ts)
    struct LightColors {
        // Primary colors
        static constexpr const char* primaryDefault = "#18a058";
        static constexpr const char* primaryHover = "#36ad6a";
        static constexpr const char* primaryActive = "#0c7a43";
        static constexpr const char* primarySuppl = "#36ad6a";

        // Info colors
        static constexpr const char* infoDefault = "#2080f0";
        static constexpr const char* infoHover = "#4098fc";
        static constexpr const char* infoActive = "#1060c9";
        static constexpr const char* infoSuppl = "#4098fc";

        // Error colors
        static constexpr const char* errorDefault = "#d03050";
        static constexpr const char* errorHover = "#de576d";
        static constexpr const char* errorActive = "#ab1f3f";
        static constexpr const char* errorSuppl = "#de576d";

        // Warning colors
        static constexpr const char* warningDefault = "#f0a020";
        static constexpr const char* warningHover = "#fcb040";
        static constexpr const char* warningActive = "#c97c10";
        static constexpr const char* warningSuppl = "#fcb040";

        // Success colors (same as primary)
        static constexpr const char* successDefault = "#18a058";
        static constexpr const char* successHover = "#36ad6a";
        static constexpr const char* successActive = "#0c7a43";
        static constexpr const char* successSuppl = "#36ad6a";

        // Text colors
        static constexpr const char* textColor1 = "rgb(31, 34, 37)";
        static constexpr const char* textColor2 = "rgb(51, 54, 57)";
        static constexpr const char* textColor3 = "rgb(118, 124, 130)";

        // Border and background colors
        static constexpr const char* borderColor = "rgb(224, 224, 230)";
        static constexpr const char* dividerColor = "rgb(239, 239, 245)";
        static constexpr const char* inputColor = "rgba(0, 0, 0, 0)";
        static constexpr const char* inputColorDisabled = "rgb(250, 250, 252)";
        static constexpr const char* actionColor = "rgb(250, 250, 252)";
        static constexpr const char* hoverColor = "rgb(243, 243, 245)";
        static constexpr const char* pressedColor = "rgb(237, 237, 239)";

        // Naive UI background colors
        static constexpr const char* neutralBase = "#FFF";
        static constexpr const char* neutralBody = "#fff";
        static constexpr const char* neutralCard = "#fff";
        static constexpr const char* neutralPopover = "#fff";
        static constexpr const char* neutralModal = "#fff";

        // Alpha values for transparent colors
        static constexpr const char* placeholderColor = "rgba(0, 0, 0, 0.24)";
        static constexpr const char* disabledColor = "rgba(0, 0, 0, 0.5)";
        static constexpr const char* primaryColorAlpha20 = "rgba(24, 160, 88, 0.2)";
        static constexpr const char* actionColorAlpha02 = "rgba(0, 0, 0, 0.02)";

        // Button specific colors
        static constexpr const char* buttonColor2 = "rgba(46, 51, 56, .05)";
        static constexpr const char* buttonColor2Hover = "rgba(46, 51, 56, .09)";
        static constexpr const char* buttonColor2Pressed = "rgba(46, 51, 56, .13)";
    };

    // 暗色主题颜色 (来自 naive-ui/src/_styles/common/dark.ts)
    struct DarkColors {
        // Primary colors
        static constexpr const char* primaryDefault = "#63e2b7";
        static constexpr const char* primaryHover = "#7fe7c4";
        static constexpr const char* primaryActive = "#5acea7";
        static constexpr const char* primarySuppl = "rgb(42, 148, 125)";

        // Info colors
        static constexpr const char* infoDefault = "#70c0e8";
        static constexpr const char* infoHover = "#8acbec";
        static constexpr const char* infoActive = "#66afd3";
        static constexpr const char* infoSuppl = "rgb(56, 137, 197)";

        // Error colors
        static constexpr const char* errorDefault = "#e88080";
        static constexpr const char* errorHover = "#e98b8b";
        static constexpr const char* errorActive = "#e57272";
        static constexpr const char* errorSuppl = "rgb(208, 58, 82)";

        // Warning colors
        static constexpr const char* warningDefault = "#f2c97d";
        static constexpr const char* warningHover = "#f5d599";
        static constexpr const char* warningActive = "#e6c260";
        static constexpr const char* warningSuppl = "rgb(240, 138, 0)";

        // Success colors (same as primary)
        static constexpr const char* successDefault = "#63e2b7";
        static constexpr const char* successHover = "#7fe7c4";
        static constexpr const char* successActive = "#5acea7";
        static constexpr const char* successSuppl = "rgb(42, 148, 125)";

        // Text colors (calculated from overlay function in dark.ts)
        static constexpr const char* textColor1 = "rgba(255, 255, 255, 0.9)";
        static constexpr const char* textColor2 = "rgba(255, 255, 255, 0.82)";
        static constexpr const char* textColor3 = "rgba(255, 255, 255, 0.52)";

        // Border and background colors (calculated from overlay function)
        static constexpr const char* borderColor = "rgba(255, 255, 255, 0.24)";
        static constexpr const char* dividerColor = "rgba(255, 255, 255, 0.09)";
        static constexpr const char* inputColor = "rgba(255, 255, 255, 0.1)";
        static constexpr const char* inputColorDisabled = "rgba(255, 255, 255, 0.06)";
        static constexpr const char* actionColor = "rgba(255, 255, 255, 0.06)";
        static constexpr const char* hoverColor = "rgba(255, 255, 255, 0.09)";
        static constexpr const char* pressedColor = "rgba(255, 255, 255, 0.05)";

        // Naive UI background colors (from dark.ts)
        static constexpr const char* neutralBase = "#000";
        static constexpr const char* neutralBody = "rgb(16, 16, 20)";
        static constexpr const char* neutralCard = "rgb(24, 24, 28)";
        static constexpr const char* neutralPopover = "rgb(72, 72, 78)";
        static constexpr const char* neutralModal = "rgb(44, 44, 50)";

        // Alpha values for transparent colors
        static constexpr const char* placeholderColor = "rgba(255, 255, 255, 0.38)";
        static constexpr const char* disabledColor = "rgba(255, 255, 255, 0.38)";
        static constexpr const char* primaryColorAlpha20 = "rgba(99, 226, 183, 0.2)";
        static constexpr const char* actionColorAlpha02 = "rgba(255, 255, 255, 0.02)";

        // Button specific colors
        static constexpr const char* buttonColor2 = "rgba(255, 255, 255, .08)";
        static constexpr const char* buttonColor2Hover = "rgba(255, 255, 255, .12)";
        static constexpr const char* buttonColor2Pressed = "rgba(255, 255, 255, .08)";
    };
}

class GTheme : public QObject
{
    Q_OBJECT

  public:
    enum class Mode
    {
        Light,
        Dark
    };

    struct Palette
    {
        QColor background;
        QColor surface;
        QColor border;
        QColor primary;
        QColor primaryHover;
        QColor text;
        QColor textSecondary;
        QColor textMuted;
        QColor success;
    };

    static GTheme& instance();

    void setMode(Mode mode);
    [[nodiscard]] Mode mode() const
    {
        return m_mode;
    }
    [[nodiscard]] const Palette& palette() const
    {
        return m_palette;
    }

    // Apply global app stylesheet covering all components
    void applyToApp();

  signals:
    void themeChanged();

  private:
    explicit GTheme(QObject* parent = nullptr);
    void updatePalette();
    [[nodiscard]] QString buildGlobalStyleSheet() const;

    // Modular stylesheet builders
    [[nodiscard]] QString buildBaseStyles() const;
    [[nodiscard]] QString buildLabelStyles() const;
    [[nodiscard]] QString buildButtonStyles() const;
    [[nodiscard]] QString buildInputStyles() const;
    [[nodiscard]] QString buildScrollAreaStyles() const;
    [[nodiscard]] QString buildListStyles() const;
    [[nodiscard]] QString buildConnectionStyles() const;
    [[nodiscard]] QString buildSeparatorStyles() const;

    Mode m_mode{Mode::Light};
    Palette m_palette{};
    Palette m_light{};
    Palette m_dark{};
};
