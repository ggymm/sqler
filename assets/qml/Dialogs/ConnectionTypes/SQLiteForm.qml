// SQLite 连接配置页面（选择数据库文件路径）
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

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: root.theme.spacingLarge
            spacing: root.theme.spacingNormal

            Label {
                Layout.alignment: Qt.AlignHCenter
                color: root.theme.textPrimary
                font.bold: true
                font.pixelSize: root.theme.fontSizeTitle
                text: "SQLite 连接配置"
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
                    placeholderText: "例如：本地SQLite"
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
                    text: "数据库文件路径"
                }

                TextField {
                    id: pathField

                    Layout.fillWidth: true
                    Layout.preferredWidth: root.theme.formInputWidth
                    color: root.theme.inputTextColor
                    placeholderText: "输入 .db/.sqlite 文件路径"
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

            Label {
                color: root.theme.textSecondary
                font.pixelSize: root.theme.fontSizeSmall
                text: "提示：暂未提供文件选择对话框，请直接输入文件绝对路径。"
            }
        }
    }
}
