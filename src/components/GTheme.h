#pragma once

#include <QColor>
#include <QObject>

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

    Mode m_mode{Mode::Light};
    Palette m_palette{};
    Palette m_light{};
    Palette m_dark{};
};
