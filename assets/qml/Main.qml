pragma ComponentBehavior: Bound
// 主界面（QML）
// - 顶部 48px 的空白栏
// - 左侧：连接管理列表（右键菜单：新建/刷新；项：打开/测试/编辑/删除）
// - 右侧：Tab 视图，默认"表"页；打开连接后新增"表 - <连接名>"页
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "Dialogs" as Dialogs
import "Views" as Views

ApplicationWindow {
    id: win
    // Qualified backend reference for strict lint
    property var backendRef: backend

    function deleteConn(c: var): void {
        if (!c || !c.id)
            return;
        if (Qt.platform.os === "android" || Qt.platform.os === "ios") {
            // inline confirm fallback
            win.backendRef.deleteConnection(c.id);
            win.backendRef.refreshConnections();
            return;
        }
        // Simple JS confirm style using Dialog
        confirmDialog.message = `确定删除连接 '${c.name || c.id}' 吗？`;
        confirmDialog.onAccept = function () {
            win.backendRef.deleteConnection(c.id);
            win.backendRef.refreshConnections();
        };
        confirmDialog.open();
    }

    function editConn(c: var): void {
        newConnDialog.openForEdit(c);
    }

    function newConn(): void {
        newConnDialog.openForNew();
    }

    function newQuery(): void {
        // TODO: 实现新建查询功能
        infoDialog.show("新建查询功能开发中...");
    }

    // 打开连接：在右侧新增一个"表 - 名称"的 Tab，展示该连接下的表名
    function openConn(c: var): void {
        if (!c)
            return;
        // Create a new tab showing tables
        var page = tablesComponent.createObject(stackLayout, { connection: c, backendRef: win.backendRef });

        // Add new tab button
        var tabButton = tabButtonComponent.createObject(tabBar);
        tabButton.text = `表 - ${c.name || c.id}`;

        // Set current index to the new tab
        tabBar.currentIndex = tabBar.count - 1;
    }

    function showFunctions(): void {
        // TODO: 实现显示函数功能
        infoDialog.show("显示函数功能开发中...");
    }

    function showQueries(): void {
        // TODO: 实现显示查询功能
        infoDialog.show("显示查询功能开发中...");
    }

    function showTables(): void {
        // TODO: 实现显示表功能
        infoDialog.show("显示表功能开发中...");
    }

    function showUsers(): void {
        // TODO: 实现显示用户功能
        infoDialog.show("显示用户功能开发中...");
    }

    function testConn(c: var): void {
        const res = win.backendRef.testConnection(c);
        if (res.ok)
            infoDialog.show("连接成功");
        else
            infoDialog.show("测试连接失败: " + (res.error || ""));
    }

    // 应用整体背景色
    color: theme.backgroundColor
    height: theme.windowHeight
    title: "SQLER"
    visible: true
    width: theme.windowWidth

    // 顶部栏：工具栏 + 标题
    header: Column {
        spacing: 0

        // 工具栏
        Rectangle {
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
            color: theme.surfaceColor
            height: theme.toolbarHeight
            width: parent.width

            Row {
                anchors.left: parent.left
                anchors.leftMargin: theme.spacingNormal
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.spacingSmall

                // 新建连接
                Loader {
                    sourceComponent: toolbarButton

                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/new-conn.svg";
                        item.buttonText = "新建连接";
                        item.clickHandler = newConn;
                    }
                }

                // 新建查询
                Loader {
                    sourceComponent: toolbarButton

                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/new-query.svg";
                        item.buttonText = "新建查询";
                        item.clickHandler = newQuery;
                    }
                }

                // 分割线
                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    color: theme.dividerColor
                    height: theme.toolbarSeparatorHeight
                    width: theme.toolbarSeparatorWidth
                }

                // 表
                Loader {
                    sourceComponent: toolbarButton

                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/table.svg";
                        item.buttonText = "表";
                        item.clickHandler = showTables;
                    }
                }

                // 查询
                Loader {
                    sourceComponent: toolbarButton

                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/query.svg";
                        item.buttonText = "查询";
                        item.clickHandler = showQueries;
                    }
                }

                // 函数
                Loader {
                    sourceComponent: toolbarButton

                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/function.svg";
                        item.buttonText = "函数";
                        item.clickHandler = showFunctions;
                    }
                }

                // 用户
                Loader {
                    sourceComponent: toolbarButton

                    onLoaded: {
                        item.iconSource = "qrc:/assets/icons/user.svg";
                        item.buttonText = "用户";
                        item.clickHandler = showUsers;
                    }
                }
            }
        }

        // 标题栏
        Rectangle {
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
            color: theme.backgroundColor
            height: theme.titleBarHeight
            width: parent.width

            Label {
                anchors.centerIn: parent
                color: theme.textPrimary
                font.bold: true
                font.pixelSize: theme.fontSizeNormal
                text: "数据库连接管理"
            }
        }
    }

    // 菜单栏
    menuBar: MenuBar {
        background: Rectangle {
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
            color: theme.surfaceColor
        }

        Menu {
            title: "文件"

            background: Rectangle {
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                color: theme.surfaceColor
                radius: theme.radiusNormal
            }

            MenuItem {
                id: miFileNew
                text: "新建连接"

                background: Rectangle {
                    color: miFileNew.hovered ? theme.hoverColor : "transparent"
                    radius: theme.radiusSmall
                }
                contentItem: Label {
                    color: theme.textPrimary
                    text: miFileNew.text
                }

                onTriggered: newConn()
            }

            MenuSeparator {
                background: Rectangle {
                    color: theme.dividerColor
                    height: 1
                }
            }

            MenuItem {
                id: miFileQuit
                text: "退出"

                background: Rectangle {
                    color: miFileQuit.hovered ? theme.hoverColor : "transparent"
                    radius: theme.radiusSmall
                }
                contentItem: Label {
                    color: theme.textPrimary
                    text: miFileQuit.text
                }

                onTriggered: Qt.quit()
            }
        }

        Menu {
            title: "视图"

            background: Rectangle {
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                color: theme.surfaceColor
                radius: theme.radiusNormal
            }

            MenuItem {
                id: miToggleTheme
                text: theme.isDarkMode ? "切换到亮色主题" : "切换到暗色主题"

                background: Rectangle {
                    color: miToggleTheme.hovered ? theme.hoverColor : "transparent"
                    radius: theme.radiusSmall
                }
                contentItem: Label {
                    color: theme.textPrimary
                    text: miToggleTheme.text
                }

                onTriggered: theme.toggleTheme()
            }
        }

        Menu {
            title: "帮助"

            background: Rectangle {
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                color: theme.surfaceColor
                radius: theme.radiusNormal
            }

            MenuItem {
                id: miAbout
                text: "关于"

                background: Rectangle {
                    color: miAbout.hovered ? theme.hoverColor : "transparent"
                    radius: theme.radiusSmall
                }
                contentItem: Label {
                    color: theme.textPrimary
                    text: miAbout.text
                }

                onTriggered: aboutDialog.open()
            }
        }
    }

    // 主题系统
    Theme {
        id: theme

    }

    // 工具栏按钮组件
    Component {
        id: toolbarButton

        ItemDelegate {
            id: tb
            property string buttonText
            property var clickHandler
            property string iconSource

            height: theme.toolbarButtonHeight
            width: theme.toolbarButtonWidth

            background: Rectangle {
                color: tb.hovered ? theme.toolbarHoverColor : "transparent"
                radius: theme.radiusSmall
            }
            contentItem: Column {
                anchors.centerIn: parent
                spacing: theme.spacingTiny

                Image {
                    anchors.horizontalCenter: parent.horizontalCenter
                    height: theme.iconSizeToolbar
                    source: iconSource
                    width: theme.iconSizeToolbar
                }

                Label {
                    anchors.horizontalCenter: parent.horizontalCenter
                    color: theme.textSecondary
                    font.pixelSize: theme.fontSizeSmall
                    horizontalAlignment: Text.AlignHCenter
                    text: buttonText
                }
            }

            onClicked: if (clickHandler)
            clickHandler()
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
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                color: theme.surfaceColor
            }
            contentItem: ColumnLayout {
                spacing: 0

                Label {
                    color: theme.textPrimary
                    font.bold: true
                    font.pixelSize: theme.fontSizeLarge
                    padding: theme.spacingSmall
                    text: "连接管理"
                }

                Rectangle {
                    Layout.fillWidth: true
                    color: theme.dividerColor
                    Layout.preferredHeight: 1
                }

                ListView {
                    id: list

                    Layout.fillHeight: true
                    Layout.fillWidth: true
                    Layout.topMargin: theme.spacingSmall
                    clip: true
                    model: win.backendRef.connections

                    ScrollBar.vertical: ScrollBar {
                        background: Rectangle {
                            color: theme.backgroundColor
                        }
                        contentItem: Rectangle {
                            color: theme.borderColor
                            radius: width / 2
                        }
                    }

                    // 列表项委托：显示连接名并附带右键菜单
                    delegate: ItemDelegate {
                        id: connItem
                        property var conn: modelData

                        text: (modelData.name || modelData.id)
                        width: ListView.view.width

                        background: Rectangle {
                            color: connItem.hovered ? theme.hoverColor : connItem.pressed
                                                     ? theme.pressedColor : "transparent"
                            radius: theme.radiusSmall
                        }
                        contentItem: Label {
                            color: theme.textPrimary
                            font.pixelSize: theme.fontSizeNormal
                            leftPadding: theme.spacingNormal
                            text: connItem.text
                            verticalAlignment: Text.AlignVCenter
                        }

                        onClicked: openConn(modelData)
                        onPressAndHold: connMenu.open()

                        Menu {
                            id: connMenu

                            background: Rectangle {
                                border.color: theme.borderColor
                                border.width: theme.menuBarBorderWidth
                                color: theme.surfaceColor
                                radius: theme.radiusNormal
                            }

                            MenuItem {
                                id: miOpen
                                text: "打开"

                                background: Rectangle {
                                    color: miOpen.hovered ? theme.hoverColor : "transparent"
                                    radius: theme.radiusSmall
                                }
                                contentItem: Label {
                                    color: theme.textPrimary
                                    text: miOpen.text
                                }

                                onTriggered: openConn(conn)
                            }

                            MenuItem {
                                id: miTest
                                text: "测试连接"

                                background: Rectangle {
                                    color: miTest.hovered ? theme.hoverColor : "transparent"
                                    radius: theme.radiusSmall
                                }
                                contentItem: Label {
                                    color: theme.textPrimary
                                    text: miTest.text
                                }

                                onTriggered: testConn(conn)
                            }

                            MenuItem {
                                id: miEdit
                                text: "编辑"

                                background: Rectangle {
                                    color: miEdit.hovered ? theme.hoverColor : "transparent"
                                    radius: theme.radiusSmall
                                }
                                contentItem: Label {
                                    color: theme.textPrimary
                                    text: miEdit.text
                                }

                                onTriggered: editConn(conn)
                            }

                            MenuItem {
                                id: miDelete
                                text: "删除"

                                background: Rectangle {
                                    color: miDelete.hovered ? theme.hoverColor : "transparent"
                                    radius: theme.radiusSmall
                                }
                                contentItem: Label {
                                    color: theme.textPrimary
                                    text: miDelete.text
                                }

                                onTriggered: deleteConn(conn)
                            }
                        }

                        TapHandler {
                            acceptedButtons: Qt.RightButton

                            onTapped: connMenu.open()
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
                        border.color: theme.borderColor
                        border.width: theme.menuBarBorderWidth
                        color: theme.surfaceColor
                        radius: theme.radiusNormal
                    }

                    MenuItem {
                        text: "新建连接"

                        background: Rectangle {
                            border.color: control.hovered ? theme.primaryColorLight : "transparent"
                            border.width: control.hovered ? 1 : 0
                            color: control.hovered ? theme.toolbarHoverColor : "transparent"
                            radius: theme.radiusSmall
                        }
                        contentItem: Label {
                            color: theme.textPrimary
                            text: control.text
                        }

                        onTriggered: newConn()
                    }

                    MenuItem {
                        text: "刷新"

                        background: Rectangle {
                            border.color: control.hovered ? theme.primaryColorLight : "transparent"
                            border.width: control.hovered ? 1 : 0
                            color: control.hovered ? theme.toolbarHoverColor : "transparent"
                            radius: theme.radiusSmall
                        }
                        contentItem: Label {
                            color: theme.textPrimary
                            text: control.text
                        }

                        onTriggered: win.backendRef.refreshConnections()
                    }
                }
            }
        }

        // 右侧：Tab 区域。默认提供一个"表"页占位
        Pane {
            id: rightPane

            SplitView.fillWidth: true

            background: Rectangle {
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                color: theme.surfaceColor
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
                            color: theme.dividerColor
                            height: 1
                            width: parent.width
                        }
                    }

                    TabButton {
                        text: "表"

                        background: Rectangle {
                            border.color: control.checked ? theme.dividerColor : "transparent"
                            border.width: theme.menuBarBorderWidth
                            color: control.checked ? theme.surfaceColor : control.hovered
                                                     ? theme.hoverColor : "transparent"
                            radius: theme.radiusSmall

                            Rectangle {
                                anchors.bottom: parent.bottom
                                color: theme.primaryColor
                                height: 2
                                visible: control.checked
                                width: parent.width
                            }
                        }
                        contentItem: Label {
                            color: control.checked ? theme.textPrimary : theme.textSecondary
                            font.pixelSize: theme.fontSizeNormal
                            horizontalAlignment: Text.AlignHCenter
                            text: control.text
                            verticalAlignment: Text.AlignVCenter
                        }
                    }
                }

                StackLayout {
                    id: stackLayout

                    Layout.fillHeight: true
                    Layout.fillWidth: true
                    currentIndex: tabBar.currentIndex

                    // Placeholder page
                    Rectangle {
                        color: theme.backgroundColor
                    }
                }
            }
        }
    }

    // 弹窗与复用组件
    Dialogs.NewConnectionDialog {
        id: newConnDialog

        parentWindow: win
        theme: theme
        backendRef: win.backendRef
    }

    // Simple confirm/info helpers
    Component {
        id: infoDialogComponent

        Dialog {
            id: d

            property alias text: msg.text

            modal: true
            standardButtons: Dialog.Ok
            title: "信息"

            background: Rectangle {
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                color: theme.surfaceColor
                radius: theme.radiusNormal
            }
            contentItem: Label {
                id: msg

                color: theme.textPrimary
                padding: theme.spacingLarge
                text: ""
                wrapMode: Text.WordWrap
            }
            header: Rectangle {
                border.color: theme.borderColor
                border.width: theme.menuBarBorderWidth
                color: theme.surfaceColor
                height: 48
                radius: theme.radiusNormal

                Label {
                    anchors.centerIn: parent
                    color: theme.textPrimary
                    font.bold: true
                    font.pixelSize: theme.fontSizeLarge
                    text: d.title
                }
            }
        }
    }

    QtObject {
        id: infoDialog

        function show(t) {
            var d = infoDialogComponent.createObject(win);
            d.text = t;
            d.open();
        }
    }

    Dialog {
        id: confirmDialog

        property string message: ""
        property var onAccept: function () {}

        modal: true
        standardButtons: Dialog.Ok | Dialog.Cancel
        title: "确认"

        background: Rectangle {
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
            color: theme.surfaceColor
            radius: theme.radiusNormal
        }
        contentItem: Label {
            color: theme.textPrimary
            padding: theme.spacingLarge
            text: confirmDialog.message
            wrapMode: Text.WordWrap
        }
        header: Rectangle {
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
            color: theme.surfaceColor
            height: 48
            radius: theme.radiusNormal

            Label {
                anchors.centerIn: parent
                color: theme.textPrimary
                font.bold: true
                font.pixelSize: theme.fontSizeLarge
                text: confirmDialog.title
            }
        }

        onAccepted: onAccept()
    }

    // 表页组件工厂
    Component {
        id: tablesComponent

        Views.TablesPane {
        }
    }

    // TabButton 组件工厂
    Component {
        id: tabButtonComponent

        // text property will be set after creation
        TabButton {
        }
    }

    // 关于对话框
    Dialog {
        id: aboutDialog

        modal: true
        standardButtons: Dialog.Ok
        title: "关于 SQLER"

        background: Rectangle {
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
            color: theme.surfaceColor
            radius: theme.radiusNormal
        }
        contentItem: Column {
            padding: theme.spacingLarge
            spacing: theme.spacingNormal

            Label {
                anchors.horizontalCenter: parent.horizontalCenter
                color: theme.textPrimary
                font.bold: true
                font.pixelSize: theme.fontSizeHeading
                text: "SQLER"
            }

            Label {
                anchors.horizontalCenter: parent.horizontalCenter
                color: theme.textSecondary
                text: "版本 1.0.0"
            }

            Rectangle {
                color: theme.dividerColor
                height: 1
                width: parent.width
            }

            Label {
                anchors.horizontalCenter: parent.horizontalCenter
                color: theme.textPrimary
                horizontalAlignment: Text.AlignHCenter
                text: "一个现代化的数据库管理工具\n支持 MySQL、Redis 等数据库"
                width: parent.width - 2 * theme.spacingLarge
                wrapMode: Text.WordWrap
            }
        }
        header: Rectangle {
            border.color: theme.borderColor
            border.width: theme.menuBarBorderWidth
            color: theme.surfaceColor
            height: 48
            radius: theme.radiusNormal

            Label {
                anchors.centerIn: parent
                color: theme.textPrimary
                font.bold: true
                font.pixelSize: theme.fontSizeLarge
                text: aboutDialog.title
            }
        }
    }
}
