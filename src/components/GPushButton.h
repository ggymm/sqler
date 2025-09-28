#pragma once

#include <QPushButton>

class GPushButton : public QPushButton {
    Q_OBJECT

public:
    enum class Variant {
        Primary,
        Secondary,
        Dialog,
        Toolbar,
        Neutral
    };

    explicit GPushButton(QWidget* parent = nullptr);
    explicit GPushButton(const QString& text, QWidget* parent = nullptr);
    GPushButton(const QString& text, Variant variant, QWidget* parent = nullptr);

    void setVariant(Variant variant);
    [[nodiscard]] Variant variant() const { return m_variant; }

private:
    void setup();
    void applyStyle();

    Variant m_variant = Variant::Primary;
};

