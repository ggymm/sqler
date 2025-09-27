#pragma once

#include <QPushButton>

class ThemedButton : public QPushButton {
    Q_OBJECT

public:
    enum class Variant {
        Primary,
        Secondary,
        Dialog
    };

    explicit ThemedButton(QWidget* parent = nullptr);
    explicit ThemedButton(const QString& text, QWidget* parent = nullptr);
    ThemedButton(const QString& text, Variant variant, QWidget* parent = nullptr);

    void setVariant(Variant variant);
    [[nodiscard]] Variant variant() const { return m_variant; }

private slots:
    void onThemeChanged();

private:
    void setupButton();
    void applyTheme();

    Variant m_variant = Variant::Primary;
};