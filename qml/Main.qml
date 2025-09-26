// 主界面（QML）
// - 顶部 48px 的空白栏
// - 左侧：连接管理列表（右键菜单：新建/刷新；项：打开/测试/编辑/删除）
// - 右侧：Tab 视图，默认"表"页；打开连接后新增"表 - <连接名>"页
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import SQLER 1.0
import "Dialogs" as Dialogs
import "Views" as Views

ApplicationWindow {
    id: win
    visible: true
    width: theme.windowWidth
    height: theme.windowHeight
    title: "SQLER"

    // 主题系统
    Theme { id: theme }

    // 工具栏按钮组件
    Component {
        id: toolbarButton

        ItemDelegate {
            property string iconSource
            property string buttonText
            property var clickHandler

            width: theme.toolbarButtonWidth
            height: theme.toolbarButtonHeight

            background: Rectangle {
                color: parent.hovered ? theme.toolbarHoverColor : "transparent"
                radius: theme.radiusSmall
            }

            onClicked: if (clickHandler) clickHandler()

            contentItem: Column {
                anchors.centerIn: parent
                spacing: theme.spacingTiny

                Image {
                    source: iconSource
                    width: theme.iconSizeToolbar
                    height: theme.iconSizeToolbar
                    anchors.horizontalCenter: parent.horizontalCenter
                }

                Label {
                    text: buttonText
                    color: theme.textSecondary
                    font.pixelSize: theme.fontSizeSmall
                    anchors.horizontalCenter: parent.horizontalCenter
                    horizontalAlignment: Text.AlignHCenter
                }
            }
        }
    }

    // 应用整体背景色
    color: theme.backgroundColor

    // 菜单栏
    menuBar: MenuBar {
        background: Rectangle {
            color: theme.surfaceColor
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
        }

        Menu {
            title: "文件"
            background: Rectangle {
                color: theme.surfaceColor
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                radius: theme.radiusNormal
            }

            MenuItem {
                text: "新建连接"
                onTriggered: newConn()
                background: Rectangle {
                    color: parent.hovered ? theme.hoverColor : "transparent"
                    radius: theme.radiusSmall
                }
                contentItem: Label {
                    text: parent.text
                    color: theme.textPrimary
                }
            }
            MenuSeparator {
                background: Rectangle {
                    color: theme.dividerColor
                    height: 1
                }
            }
            MenuItem {
                text: "退出"
                onTriggered: Qt.quit()
                background: Rectangle {
                    color: parent.hovered ? theme.hoverColor : "transparent"
                    radius: theme.radiusSmall
                }
                contentItem: Label {
                    text: parent.text
                    color: theme.textPrimary
                }
            }
        }

        Menu {
            title: "视图"
            background: Rectangle {
                color: theme.surfaceColor
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                radius: theme.radiusNormal
            }

            MenuItem {
                text: theme.isDarkMode ? "切换到亮色主题" : "切换到暗色主题"
                onTriggered: theme.toggleTheme()
                background: Rectangle {
                    color: parent.hovered ? theme.hoverColor : "transparent"
                    radius: theme.radiusSmall
                }
                contentItem: Label {
                    text: parent.text
                    color: theme.textPrimary
                }
            }
        }

        Menu {
            title: "帮助"
            background: Rectangle {
                color: theme.surfaceColor
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                radius: theme.radiusNormal
            }

            MenuItem {
                text: "关于"
                onTriggered: aboutDialog.open()
                background: Rectangle {
                    color: parent.hovered ? theme.hoverColor : "transparent"
                    radius: theme.radiusSmall
                }
                contentItem: Label {
                    text: parent.text
                    color: theme.textPrimary
                }
            }
        }
    }

    // 顶部栏：工具栏 + 标题
    header: Column {
        spacing: 0

        // 工具栏
        Rectangle {
            width: parent.width
            height: theme.toolbarHeight
            color: theme.surfaceColor
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth

            Row {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: theme.spacingNormal
                spacing: theme.spacingSmall

                // 新建连接
                Loader {
                    sourceComponent: toolbarButton
                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/new-conn.svg"
                        item.buttonText = "新建连接"
                        item.clickHandler = newConn
                    }
                }

                // 新建查询
                Loader {
                    sourceComponent: toolbarButton
                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/new-query.svg"
                        item.buttonText = "新建查询"
                        item.clickHandler = newQuery
                    }
                }

                // 分割线
                Rectangle {
                    width: theme.toolbarSeparatorWidth
                    height: theme.toolbarSeparatorHeight
                    color: theme.dividerColor
                    anchors.verticalCenter: parent.verticalCenter
                }

                // 表
                Loader {
                    sourceComponent: toolbarButton
                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/table.svg"
                        item.buttonText = "表"
                        item.clickHandler = showTables
                    }
                }

                // 查询
                Loader {
                    sourceComponent: toolbarButton
                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/query.svg"
                        item.buttonText = "查询"
                        item.clickHandler = showQueries
                    }
                }

                // 函数
                Loader {
                    sourceComponent: toolbarButton
                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/function.svg"
                        item.buttonText = "函数"
                        item.clickHandler = showFunctions
                    }
                }

                // 用户
                Loader {
                    sourceComponent: toolbarButton
                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/user.svg"
                        item.buttonText = "用户"
                        item.clickHandler = showUsers
                    }
                }
            }
        }

        // 标题栏
        Rectangle {
            width: parent.width
            height: theme.titleBarHeight
            color: theme.backgroundColor
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth

            Label {
                text: "数据库连接管理"
                color: theme.textPrimary
                font.pixelSize: theme.fontSizeNormal
                font.bold: true
                anchors.centerIn: parent
            }
        }
    }

    SplitView {
        id: split
        anchors.fill: parent

        // 左侧：连接管理区
        Pane {
            id: leftPane
            SplitView.preferredWidth: 280

            background: Rectangle {
                color: theme.surfaceColor
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
            }

            contentItem: ColumnLayout {
                spacing: 0

                Label {
                    text: "连接管理"
                    padding: theme.spacingSmall
                    font.bold: true
                    font.pixelSize: theme.fontSizeLarge
                    color: theme.textPrimary
                }

                Rectangle {
                    Layout.fillWidth: true
                    height: 1
                    color: theme.dividerColor
                }

                ListView {
                    id: list
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.topMargin: theme.spacingSmall
                    clip: true
                    model: backend.connections

                    // 列表项委托：显示连接名并附带右键菜单
                    delegate: ItemDelegate {
                        width: ListView.view.width
                        text: (modelData.name || modelData.id)

                        background: Rectangle {
                            color: parent.hovered ? theme.hoverColor :
                                   parent.pressed ? theme.pressedColor : "transparent"
                            radius: theme.radiusSmall
                        }

                        contentItem: Label {
                            text: parent.text
                            color: theme.textPrimary
                            font.pixelSize: theme.fontSizeNormal
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: theme.spacingNormal
                        }

                        onClicked: openConn(modelData)
                        onPressAndHold: connMenu.open()
                        property var conn: modelData

                        Menu {
                            id: connMenu

                            background: Rectangle {
                                color: theme.surfaceColor
                                border.color: theme.borderColor
                                border.width: theme.menuBarBorderWidth
                                radius: theme.radiusNormal
                            }

                            MenuItem {
                                text: "打开"
                                onTriggered: openConn(conn)
                                background: Rectangle {
                                    color: parent.hovered ? theme.hoverColor : "transparent"
                                    radius: theme.radiusSmall
                                }
                                contentItem: Label {
                                    text: parent.text
                                    color: theme.textPrimary
                                }
                            }
                            MenuItem {
                                text: "测试连接"
                                onTriggered: testConn(conn)
                                background: Rectangle {
                                    color: parent.hovered ? theme.hoverColor : "transparent"
                                    radius: theme.radiusSmall
                                }
                                contentItem: Label {
                                    text: parent.text
                                    color: theme.textPrimary
                                }
                            }
                            MenuItem {
                                text: "编辑"
                                onTriggered: editConn(conn)
                                background: Rectangle {
                                    color: parent.hovered ? theme.hoverColor : "transparent"
                                    radius: theme.radiusSmall
                                }
                                contentItem: Label {
                                    text: parent.text
                                    color: theme.textPrimary
                                }
                            }
                            MenuItem {
                                text: "删除"
                                onTriggered: deleteConn(conn)
                                background: Rectangle {
                                    color: parent.hovered ? theme.hoverColor : "transparent"
                                    radius: theme.radiusSmall
                                }
                                contentItem: Label {
                                    text: parent.text
                                    color: theme.textPrimary
                                }
                            }
                        }
                        TapHandler { acceptedButtons: Qt.RightButton; onTapped: connMenu.open() }
                    }

                    ScrollBar.vertical: ScrollBar {
                        background: Rectangle {
                            color: theme.backgroundColor
                        }
                        contentItem: Rectangle {
                            color: theme.borderColor
                            radius: width / 2
                        }
                    }

                    // 空白区域右键菜单：新建/刷新
                    TapHandler {
                        acceptedButtons: Qt.RightButton
                        onTapped: emptyMenu.open()
                    }
                }

                Menu {
                    id: emptyMenu

                    background: Rectangle {
                        color: theme.surfaceColor
                        border.color: theme.borderColor
                        border.width: theme.menuBarBorderWidth
                        radius: theme.radiusNormal
                    }

                    MenuItem {
                        text: "新建连接"
                        onTriggered: newConn()
                        background: Rectangle {
                            color: parent.hovered ? theme.toolbarHoverColor : "transparent"
                            radius: theme.radiusSmall
                            border.color: parent.hovered ? theme.primaryColorLight : "transparent"
                            border.width: parent.hovered ? 1 : 0
                        }
                        contentItem: Label {
                            text: parent.text
                            color: theme.textPrimary
                        }
                    }
                    MenuItem {
                        text: "刷新"
                        onTriggered: backend.refreshConnections()
                        background: Rectangle {
                            color: parent.hovered ? theme.toolbarHoverColor : "transparent"
                            radius: theme.radiusSmall
                            border.color: parent.hovered ? theme.primaryColorLight : "transparent"
                            border.width: parent.hovered ? 1 : 0
                        }
                        contentItem: Label {
                            text: parent.text
                            color: theme.textPrimary
                        }
                    }
                }
            }
        }

        // 右侧：Tab 区域。默认提供一个"表"页占位
        Pane {
            id: rightPane
            SplitView.fillWidth: true

            background: Rectangle {
                color: theme.surfaceColor
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
            }

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                TabBar {
                    id: tabBar
                    Layout.fillWidth: true

                    background: Rectangle {
                        color: theme.backgroundColor
                        Rectangle {
                            anchors.bottom: parent.bottom
                            width: parent.width
                            height: 1
                            color: theme.dividerColor
                        }
                    }

                    TabButton {
                        text: "表"

                        background: Rectangle {
                            color: parent.checked ? theme.surfaceColor :
                                   parent.hovered ? theme.hoverColor : "transparent"
                            border.color: parent.checked ? theme.dividerColor : "transparent"
                            border.width: theme.menuBarBorderWidth
                            radius: theme.radiusSmall

                            Rectangle {
                                visible: parent.parent.checked
                                anchors.bottom: parent.bottom
                                width: parent.width
                                height: 2
                                color: theme.primaryColor
                            }
                        }

                        contentItem: Label {
                            text: parent.text
                            color: parent.checked ? theme.textPrimary : theme.textSecondary
                            font.pixelSize: theme.fontSizeNormal
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                        }
                    }
                }

                StackLayout {
                    id: stackLayout
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    currentIndex: tabBar.currentIndex

                    // Placeholder page
                    Rectangle {
                        color: theme.backgroundColor
                    }
                }
            }
        }
    }

    function newConn() {
        newConnDialog.openForNew()
    }

    function newQuery() {
        // TODO: 实现新建查询功能
        infoDialog.show("新建查询功能开发中...")
    }

    function showTables() {
        // TODO: 实现显示表功能
        infoDialog.show("显示表功能开发中...")
    }

    function showQueries() {
        // TODO: 实现显示查询功能
        infoDialog.show("显示查询功能开发中...")
    }

    function showFunctions() {
        // TODO: 实现显示函数功能
        infoDialog.show("显示函数功能开发中...")
    }

    function showUsers() {
        // TODO: 实现显示用户功能
        infoDialog.show("显示用户功能开发中...")
    }

    function editConn(c) {
        newConnDialog.openForEdit(c)
    }

    function deleteConn(c) {
        if (!c || !c.id) return;
        if (Qt.platform.os === "android" || Qt.platform.os === "ios") {
            // inline confirm fallback
            backend.deleteConnection(c.id)
            backend.refreshConnections()
            return
        }
        // Simple JS confirm style using Dialog
        confirmDialog.message = `确定删除连接 '${c.name || c.id}' 吗？`
        confirmDialog.onAccept = function() {
            backend.deleteConnection(c.id)
            backend.refreshConnections()
        }
        confirmDialog.open()
    }

    // 打开连接：在右侧新增一个"表 - 名称"的 Tab，展示该连接下的表名
    function openConn(c) {
        if (!c) return
        // Create a new tab showing tables
        var page = tablesComponent.createObject(stackLayout, { connection: c })

        // Add new tab button
        var tabButton = tabButtonComponent.createObject(tabBar)
        tabButton.text = `表 - ${c.name || c.id}`

        // Set current index to the new tab
        tabBar.currentIndex = tabBar.count - 1
    }

    function testConn(c) {
        const res = backend.testConnection(c)
        if (res.ok) infoDialog.show("连接成功")
        else infoDialog.show("测试连接失败: " + (res.error || ""))
    }

    // 弹窗与复用组件
    Dialogs.NewConnectionDialog {
        id: newConnDialog
        theme: theme
    }

    // Simple confirm/info helpers
    Component {
        id: infoDialogComponent
        Dialog {
            id: d
            modal: true
            standardButtons: Dialog.Ok
            title: "信息"
            property alias text: msg.text

            background: Rectangle {
                color: theme.surfaceColor
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                radius: theme.radiusNormal
            }

            header: Rectangle {
                height: 48
                color: theme.surfaceColor
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                radius: theme.radiusNormal

                Label {
                    text: d.title
                    color: theme.textPrimary
                    font.pixelSize: theme.fontSizeLarge
                    font.bold: true
                    anchors.centerIn: parent
                }
            }

            contentItem: Label {
                id: msg
                text: ""
                color: theme.textPrimary
                padding: theme.spacingLarge
                wrapMode: Text.WordWrap
            }
        }
    }

    QtObject {
        id: infoDialog
        function show(t) {
            var d = infoDialogComponent.createObject(win)
            d.text = t
            d.open()
        }
    }

    Dialog {
        id: confirmDialog
        property string message: ""
        property var onAccept: function() {}
        modal: true
        title: "确认"
        standardButtons: Dialog.Ok | Dialog.Cancel
        onAccepted: onAccept()

        background: Rectangle {
            color: theme.surfaceColor
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
            radius: theme.radiusNormal
        }

        header: Rectangle {
            height: 48
            color: theme.surfaceColor
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
            radius: theme.radiusNormal

            Label {
                text: confirmDialog.title
                color: theme.textPrimary
                font.pixelSize: theme.fontSizeLarge
                font.bold: true
                anchors.centerIn: parent
            }
        }

        contentItem: Label {
            text: confirmDialog.message
            color: theme.textPrimary
            padding: theme.spacingLarge
            wrapMode: Text.WordWrap
        }
    }

    // 表页组件工厂
    Component { id: tablesComponent; Views.TablesPane { } }

    // TabButton 组件工厂
    Component {
        id: tabButtonComponent
        TabButton {
            // text property will be set after creation
        }
    }

    // 关于对话框
    Dialog {
        id: aboutDialog
        title: "关于 SQLER"
        modal: true
        standardButtons: Dialog.Ok

        background: Rectangle {
            color: theme.surfaceColor
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
            radius: theme.radiusNormal
        }

        header: Rectangle {
            height: 48
            color: theme.surfaceColor
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
            radius: theme.radiusNormal

            Label {
                text: aboutDialog.title
                color: theme.textPrimary
                font.pixelSize: theme.fontSizeLarge
                font.bold: true
                anchors.centerIn: parent
            }
        }

        contentItem: Column {
            spacing: theme.spacingNormal
            padding: theme.spacingLarge

            Label {
                text: "SQLER"
                font.pixelSize: theme.fontSizeHeading
                font.bold: true
                color: theme.textPrimary
                anchors.horizontalCenter: parent.horizontalCenter
            }

            Label {
                text: "版本 1.0.0"
                color: theme.textSecondary
                anchors.horizontalCenter: parent.horizontalCenter
            }

            Rectangle {
                width: parent.width
                height: 1
                color: theme.dividerColor
            }

            Label {
                text: "一个现代化的数据库管理工具\n支持 MySQL、Redis 等数据库"
                color: theme.textPrimary
                wrapMode: Text.WordWrap
                width: parent.width - 2 * theme.spacingLarge
                anchors.horizontalCenter: parent.horizontalCenter
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}
