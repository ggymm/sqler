// 主题配置系统
import QtQuick 2.15
import QtQuick.Controls 2.15

QtObject {
    id: theme

    // 当前主题模式：true = 暗色，false = 亮色
    property bool isDarkMode: false

    // 主色系
    property color primaryColor: isDarkMode ? "#2196F3" : "#1976D2"
    property color primaryColorLight: isDarkMode ? "#64B5F6" : "#42A5F5"
    property color primaryColorDark: isDarkMode ? "#1565C0" : "#0D47A1"

    // 背景颜色
    property color backgroundColor: isDarkMode ? "#121212" : "#FAFAFA"
    property color surfaceColor: isDarkMode ? "#1E1E1E" : "#FFFFFF"
    property color cardColor: isDarkMode ? "#2D2D2D" : "#FFFFFF"

    // 文本颜色
    property color textPrimary: isDarkMode ? "#FFFFFF" : "#212121"
    property color textSecondary: isDarkMode ? "#B3B3B3" : "#757575"
    property color textHint: isDarkMode ? "#666666" : "#9E9E9E"

    // 边框颜色
    property color borderColor: isDarkMode ? "#333333" : "#E0E0E0"
    property color dividerColor: isDarkMode ? "#424242" : "#BDBDBD"

    // 状态颜色
    property color successColor: isDarkMode ? "#4CAF50" : "#2E7D32"
    property color warningColor: isDarkMode ? "#FF9800" : "#F57C00"
    property color errorColor: isDarkMode ? "#F44336" : "#C62828"

    // 交互状态
    property color hoverColor: isDarkMode ? "#333333" : "#F5F5F5"
    property color pressedColor: isDarkMode ? "#404040" : "#EEEEEE"
    property color selectedColor: isDarkMode ? "#1976D2" : "#E3F2FD"

    // 工具栏悬浮效果
    property color toolbarHoverColor: isDarkMode ? "#424242" : "#E8F4FD"

    // 字体大小
    property real fontSizeSmall: 12
    property real fontSizeNormal: 14
    property real fontSizeLarge: 16
    property real fontSizeTitle: 18
    property real fontSizeHeading: 20

    // 间距
    property real spacingTiny: 4
    property real spacingSmall: 8
    property real spacingNormal: 12
    property real spacingLarge: 16
    property real spacingExtraLarge: 24

    // 圆角
    property real radiusSmall: 4
    property real radiusNormal: 6
    property real radiusLarge: 8

    // 组件尺寸
    // 应用程序窗口
    property real windowWidth: 1080
    property real windowHeight: 720

    // 工具栏
    property real toolbarHeight: 80
    property real toolbarButtonWidth: 80
    property real toolbarButtonHeight: 70
    property real toolbarSeparatorWidth: 1
    property real toolbarSeparatorHeight: 60

    // 标题栏
    property real titleBarHeight: 32

    // 图标尺寸
    property real iconSizeSmall: 16
    property real iconSizeNormal: 24
    property real iconSizeLarge: 32
    property real iconSizeToolbar: 36

    // 菜单栏
    property real menuBarBorderWidth: 1

    // 对话框
    property real dialogWidth: 520
    property real dialogHeaderHeight: 48
    property real dialogContentWidth: 500
    property real dialogContentHeight: 360
    property real dialogBorderWidth: 1
    property real dialogSpacing: 8
    property real dialogPadding: 8

    // Fallback values (无主题时的默认值)
    property color fallbackSurfaceColor: "#FFFFFF"
    property color fallbackBorderColor: "#E0E0E0"
    property color fallbackTextPrimary: "#212121"
    property color fallbackTextHint: "#9E9E9E"
    property color fallbackHoverColor: "#F5F5F5"
    property color fallbackBackgroundColor: "#FAFAFA"
    property real fallbackFontSizeLarge: 16
    property real fallbackRadiusNormal: 6
    property real fallbackRadiusSmall: 4
    property real fallbackSpacingNormal: 12
    property real fallbackDialogBorderWidth: 1
    property real fallbackDialogSpacing: 8
    property real fallbackDialogPadding: 8

    // 列表和视图
    property real listSpacing: 0
    property real listItemPadding: 8

    // 对话框特定颜色
    property color dialogBackgroundColor: isDarkMode ? "#2D2D2D" : "#F5F5F5"
    property color dialogContentBackground: isDarkMode ? "#1E1E1E" : "white"
    property color dialogBorderColor: isDarkMode ? "#333333" : "#D0D0D0"
    property color inputFieldBackground: isDarkMode ? "#333333" : "white"
    property color inputFieldBorder: isDarkMode ? "#555555" : "#D0D0D0"
    property color inputFieldActiveBorder: isDarkMode ? "#64B5F6" : primaryColor
    property color selectorItemBackground: isDarkMode ? "#2D2D2D" : "white"
    property color selectorItemHoverBackground: isDarkMode ? "#404040" : "#F0F8FF"
    property color selectorItemBorder: isDarkMode ? "#444444" : "#E0E0E0"

    // 表单组件尺寸
    property real formLabelWidth: 150
    property real formInputWidth: 250
    property real formButtonWidth: 100

    // 阴影
    property color shadowColor: isDarkMode ? "#000000" : "#000000"
    property real shadowOpacity: isDarkMode ? 0.8 : 0.2

    // 切换主题
    function toggleTheme() {
        isDarkMode = !isDarkMode
    }

    // 设置暗色主题
    function setDarkTheme() {
        isDarkMode = true
    }

    // 设置亮色主题
    function setLightTheme() {
        isDarkMode = false
    }
}