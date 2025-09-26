// MySQL连接配置页面
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root

    property var conn
    property var theme

    function focusFirst() {
        if (nameField)
            nameField.forceActiveFocus();
    }

    onVisibleChanged: if (visible && nameField)
                          nameField.forceActiveFocus()

    Rectangle {
        anchors.fill: parent
        border.color: root.theme.dialogBorderColor
        border.width: 1
        color: root.theme.dialogContentBackground
        radius: root.theme.radiusSmall

        ScrollView {
            anchors.fill: parent
            anchors.margins: root.theme.spacingLarge
            contentWidth: availableWidth

            ColumnLayout {
                spacing: root.theme.spacingNormal
                width: parent.width

                Label {
                    Layout.alignment: Qt.AlignHCenter
                    color: root.theme.textPrimary
                    font.bold: true
                    font.pixelSize: root.theme.fontSizeTitle
                    text: "MySQL 连接配置"
                }

                GridLayout {
                    Layout.fillWidth: true
                    columnSpacing: root.theme.spacingNormal
                    columns: 2
                    rowSpacing: root.theme.spacingNormal

                    Label {
                        Layout.preferredWidth: root.theme.formLabelWidth
                        color: root.theme.textPrimary
                        font.pixelSize: root.theme.fontSizeNormal
                        text: "连接名称"
                    }

                    TextField {
                        id: nameField

                        Layout.fillWidth: true
                        Layout.preferredWidth: root.theme.formInputWidth
                        color: root.theme.inputTextColor
                        placeholderText: "例如：本地MySQL"
                        placeholderTextColor: root.theme.inputPlaceholderColor
                        selectedTextColor: root.theme.inputSelectedTextColor
                        selectionColor: root.theme.inputSelectionColor
                        text: root.conn ? root.conn.name : ""

                        background: Rectangle {
                            border.color: parent.activeFocus ? root.theme.inputFieldActiveBorder :
                                                               root.theme.inputFieldBorder
                            border.width: 1
                            color: parent.activeFocus ? root.theme.backgroundColor :
                                                        root.theme.inputFieldBackground
                            radius: root.theme.radiusSmall
                        }

                        onTextChanged: if (root.conn)
                                           root.conn.name = text
                    }

                    Label {
                        Layout.preferredWidth: root.theme.formLabelWidth
                        color: root.theme.textPrimary
                        font.pixelSize: root.theme.fontSizeNormal
                        text: "主机地址"
                    }

                    TextField {
                        Layout.fillWidth: true
                        Layout.preferredWidth: root.theme.formInputWidth
                        color: root.theme.inputTextColor
                        placeholderText: "localhost"
                        placeholderTextColor: root.theme.inputPlaceholderColor
                        selectedTextColor: root.theme.inputSelectedTextColor
                        selectionColor: root.theme.inputSelectionColor
                        text: root.conn ? root.conn.host : ""

                        background: Rectangle {
                            border.color: parent.activeFocus ? root.theme.inputFieldActiveBorder :
                                                               root.theme.inputFieldBorder
                            border.width: 1
                            color: parent.activeFocus ? root.theme.backgroundColor :
                                                        root.theme.inputFieldBackground
                            radius: root.theme.radiusSmall
                        }

                        onTextChanged: if (root.conn)
                                           root.conn.host = text
                    }

                    Label {
                        Layout.preferredWidth: root.theme.formLabelWidth
                        color: root.theme.textPrimary
                        font.pixelSize: root.theme.fontSizeNormal
                        text: "端口"
                    }

                    SpinBox {
                        id: portSpin

                        Layout.fillWidth: true
                        Layout.preferredWidth: root.theme.formInputWidth
                        from: 1
                        to: 65535
                        value: root.conn ? root.conn.port : 3306

                        background: Rectangle {
                            border.color: parent.activeFocus ? root.theme.inputFieldActiveBorder : root.theme.inputFieldBorder
                            border.width: 1
                            color: parent.activeFocus ? root.theme.backgroundColor : root.theme.inputFieldBackground
                            radius: root.theme.radiusSmall
                        }

                        onValueChanged: if (root.conn)
                                            root.conn.port = value
                    }

                    Label {
                        Layout.preferredWidth: root.theme.formLabelWidth
                        color: root.theme.textPrimary
                        font.pixelSize: root.theme.fontSizeNormal
                        text: "用户名"
                    }

                    TextField {
                        Layout.fillWidth: true
                        Layout.preferredWidth: root.theme.formInputWidth
                        color: root.theme.inputTextColor
                        placeholderText: "root"
                        placeholderTextColor: root.theme.inputPlaceholderColor
                        selectedTextColor: root.theme.inputSelectedTextColor
                        selectionColor: root.theme.inputSelectionColor
                        text: root.conn ? root.conn.user : ""

                        background: Rectangle {
                            border.color: parent.activeFocus ? root.theme.inputFieldActiveBorder :
                                                               root.theme.inputFieldBorder
                            border.width: 1
                            color: parent.activeFocus ? root.theme.backgroundColor :
                                                        root.theme.inputFieldBackground
                            radius: root.theme.radiusSmall
                        }

                        onTextChanged: if (root.conn)
                                           root.conn.user = text
                    }

                    Label {
                        Layout.preferredWidth: root.theme.formLabelWidth
                        color: root.theme.textPrimary
                        font.pixelSize: root.theme.fontSizeNormal
                        text: "密码"
                    }

                    TextField {
                        Layout.fillWidth: true
                        Layout.preferredWidth: root.theme.formInputWidth
                        color: root.theme.inputTextColor
                        echoMode: TextInput.Password
                        placeholderTextColor: root.theme.inputPlaceholderColor
                        selectedTextColor: root.theme.inputSelectedTextColor
                        selectionColor: root.theme.inputSelectionColor
                        text: root.conn ? root.conn.password : ""

                        background: Rectangle {
                            border.color: parent.activeFocus ? root.theme.inputFieldActiveBorder :
                                                               root.theme.inputFieldBorder
                            border.width: 1
                            color: parent.activeFocus ? root.theme.backgroundColor :
                                                        root.theme.inputFieldBackground
                            radius: root.theme.radiusSmall
                        }

                        onTextChanged: if (root.conn)
                                           root.conn.password = text
                    }

                    Label {
                        Layout.preferredWidth: root.theme.formLabelWidth
                        color: root.theme.textPrimary
                        font.pixelSize: root.theme.fontSizeNormal
                        text: "数据库 (可选)"
                    }

                    TextField {
                        Layout.fillWidth: true
                        Layout.preferredWidth: root.theme.formInputWidth
                        color: root.theme.inputTextColor
                        placeholderText: "留空以显示所有数据库"
                        placeholderTextColor: root.theme.inputPlaceholderColor
                        selectedTextColor: root.theme.inputSelectedTextColor
                        selectionColor: root.theme.inputSelectionColor
                        text: root.conn ? root.conn.database : ""

                        background: Rectangle {
                            border.color: parent.activeFocus ? root.theme.inputFieldActiveBorder :
                                                               root.theme.inputFieldBorder
                            border.width: 1
                            color: parent.activeFocus ? root.theme.backgroundColor :
                                                        root.theme.inputFieldBackground
                            radius: root.theme.radiusSmall
                        }

                        onTextChanged: if (root.conn)
                                           root.conn.database = text
                    }
                }
            }
        }
    }
}
