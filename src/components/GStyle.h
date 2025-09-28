#pragma once

namespace GStyle
{

// Spacing + sizes used across components
struct Spacing
{
    static constexpr int xs = 4;
    static constexpr int sm = 8;
    static constexpr int md = 16;
    static constexpr int lg = 24;
    static constexpr int xl = 32;
};

struct Sizes
{
    static constexpr int topBarHeight = 48;
    static constexpr int sideBarWidth = 280;
    static constexpr int iconSize = 20;
    static constexpr int buttonHeight = 36;
    static constexpr int inputHeight = 32;
    static constexpr int borderRadius = 6;
    static constexpr int dialogButtonHeight = 70;
    static constexpr int formButtonWidth = 80;
};

} // namespace GStyle
