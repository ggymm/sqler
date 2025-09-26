// 新建/编辑连接对话框（QML）
// 使用模块化组件：DatabaseTypeSelector, MySQLForm, RedisForm
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "ConnectionTypes" as ConnectionTypes

ApplicationWindow {
    id: root
    width: theme.dialogWidth
    height: 550
    flags: Qt.Dialog
    modality: Qt.ApplicationModal
    color: theme.dialogBackgroundColor

    property var theme
    property var conn: ({
        id: "", name: "", type: "",
        host: "localhost", port: 3306,
        user: "", password: "", database: ""
    })

    property int currentStep: 0 // 0: type selection, 1: form
    // 当前已实现类型
    property var supportedTypes: ["mysql", "redis"]

    title: currentStep === 0 ? "新建连接" :
           (conn.type.toUpperCase() + " 连接配置")

    function openForNew() {
        conn = {
            id: "", name: "", type: "",
            host: "localhost", port: 3306,
            user: "", password: "", database: ""
        }
        currentStep = 0
        show()
    }

    function openForEdit(c) {
        conn = Object.assign({}, c)
        if (!conn.type) conn.type = "mysql"
        currentStep = 1
        show()
    }

    function closeDialog() {
        hide()
    }

    function handlePrevious() {
        if (currentStep > 0) {
            currentStep--
        }
    }

    function handleNext() {
        if (currentStep === 0) {
            // Need to select type first
            return
        }
        // Save connection
        const id = backend.saveConnection(conn)
        if (id && id.length > 0) backend.refreshConnections()
        closeDialog()
    }

    function handleCancel() {
        closeDialog()
    }

    function handleTest() {
        const res = backend.testConnection(conn)
        if (res.ok) infoDialog.show("连接成功")
        else infoDialog.show("测试连接失败: " + (res.error || ""))
    }

    function onTypeSelected(type) {
        console.log("Type selected:", type)
        conn.type = type
        console.log("Connection type set to:", conn.type)
        // Set default values based on type
        if (type === 'mysql') {
            conn.port = 3306; conn.user = 'root'; conn.database = ''
        } else if (type === 'redis') {
            conn.port = 6379; conn.user = ''; conn.database = '0'
        } else if (type === 'postgresql') {
            conn.port = 5432; conn.user = 'postgres'; conn.database = ''
        } else if (type === 'sqlserver') {
            conn.port = 1433; conn.user = 'sa'; conn.database = ''
        } else if (type === 'sqlite') {
            conn.port = 0; conn.user = ''; conn.database = ''
        } else if (type === 'mongodb') {
            conn.port = 27017; conn.user = ''; conn.database = ''
        } else if (type === 'oracle') {
            conn.port = 1521; conn.user = ''; conn.database = ''
        }
        currentStep = 1
        console.log("Current step set to:", currentStep)
    }

    // Main content area
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: theme.spacingNormal
        spacing: 0

        // Content area
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            // Type selection page
            ConnectionTypes.DatabaseTypeSelector {
                id: typeSelector
                anchors.fill: parent
                visible: currentStep === 0
                theme: root.theme
                conn: root.conn
                onTypeSelected: function(type) { root.onTypeSelected(type) }
            }

            // MySQL form
            ConnectionTypes.MySQLForm {
                id: mysqlForm
                anchors.fill: parent
                visible: currentStep === 1 && conn.type === "mysql"
                theme: root.theme
                conn: root.conn
            }

            // Redis form
            ConnectionTypes.RedisForm {
                id: redisForm
                anchors.fill: parent
                visible: currentStep === 1 && conn.type === "redis"
                theme: root.theme
                conn: root.conn
            }

            // Unsupported placeholder
            Item {
                id: unsupported
                anchors.fill: parent
                visible: currentStep === 1 && supportedTypes.indexOf(conn.type) === -1

                Rectangle {
                    anchors.centerIn: parent
                    width: parent.width * 0.8
                    height: 160
                    color: theme.dialogContentBackground
                    border.color: theme.dialogBorderColor
                    border.width: 1
                    radius: theme.radiusNormal

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: theme.spacingLarge
                        spacing: theme.spacingSmall
                        Label {
                            text: (conn.type || '').toUpperCase() + " 暂未支持"
                            color: theme.textPrimary
                            font.bold: true
                            font.pixelSize: theme.fontSizeTitle
                            Layout.alignment: Qt.AlignHCenter
                        }
                        Label {
                            text: "暂不支持该数据库类型的配置与连接，敬请期待。"
                            color: theme.textSecondary
                            font.pixelSize: theme.fontSizeNormal
                            wrapMode: Text.WordWrap
                            horizontalAlignment: Text.AlignHCenter
                            Layout.alignment: Qt.AlignHCenter
                        }
                    }
                }
            }
        }

        // Button area
        Rectangle {
            Layout.fillWidth: true
            height: 60
            color: theme.dialogContentBackground
            border.color: theme.dialogBorderColor
            border.width: 1

            RowLayout {
                anchors.fill: parent
                anchors.margins: theme.spacingNormal
                spacing: theme.spacingNormal

                Button {
                    text: "测试连接"
                    enabled: currentStep === 1 && supportedTypes.indexOf(conn.type) !== -1
                    Layout.preferredWidth: 100
                    onClicked: handleTest()

                    background: Rectangle {
                        color: parent.enabled ?
                               (parent.hovered ? theme.hoverColor : "transparent") :
                               theme.backgroundColor
                        border.color: theme.borderColor
                        border.width: 1
                        radius: theme.radiusSmall
                    }

                    contentItem: Text {
                        text: parent.text
                        color: parent.enabled ? theme.textPrimary : theme.textHint
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                        font.pixelSize: theme.fontSizeNormal
                    }
                }

                Item { Layout.fillWidth: true } // Spacer

                Button {
                    text: "上一步"
                    visible: currentStep > 0
                    Layout.preferredWidth: 80
                    onClicked: handlePrevious()

                    background: Rectangle {
                        color: parent.hovered ? theme.hoverColor : "transparent"
                        border.color: theme.borderColor
                        border.width: 1
                        radius: theme.radiusSmall
                    }

                    contentItem: Text {
                        text: parent.text
                        color: theme.textPrimary
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                        font.pixelSize: theme.fontSizeNormal
                    }
                }

                Button {
                    text: currentStep === 0 ? "下一步" : "确定"
                    Layout.preferredWidth: 80
                    enabled: currentStep === 0 || supportedTypes.indexOf(conn.type) !== -1
                    onClicked: {
                        if (currentStep === 1 && supportedTypes.indexOf(conn.type) === -1) {
                            infoDialog.show("该数据库类型暂未支持")
                            return
                        }
                        handleNext()
                    }

                    background: Rectangle {
                        color: parent.hovered ? theme.primaryColorLight : theme.primaryColor
                        radius: theme.radiusSmall
                    }

                    contentItem: Text {
                        text: parent.text
                        color: "white"
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                        font.pixelSize: theme.fontSizeNormal
                        font.bold: true
                    }
                }

                Button {
                    text: "取消"
                    Layout.preferredWidth: 80
                    onClicked: handleCancel()

                    background: Rectangle {
                        color: parent.hovered ? theme.hoverColor : "transparent"
                        border.color: theme.borderColor
                        border.width: 1
                        radius: theme.radiusSmall
                    }

                    contentItem: Text {
                        text: parent.text
                        color: theme.textPrimary
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                        font.pixelSize: theme.fontSizeNormal
                    }
                }
            }
        }
    }

    // Info dialog
    Dialog {
        id: infoDialog
        title: "信息"
        modal: true
        standardButtons: Dialog.Ok
        property alias text: msgLabel.text

        function show(message) {
            text = message
            open()
        }

        background: Rectangle {
            color: theme.surfaceColor
            border.color: theme.borderColor
            border.width: theme.dialogBorderWidth
            radius: theme.radiusNormal
        }

        contentItem: Label {
            id: msgLabel
            color: theme.textPrimary
            padding: theme.spacingNormal
        }
    }
}
