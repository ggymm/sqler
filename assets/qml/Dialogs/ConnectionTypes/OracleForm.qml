// Oracle 连接配置页面
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    property var theme
    property var conn
    function focusFirst() { if (nameField) nameField.forceActiveFocus() }

    Rectangle {
        anchors.fill: parent
        color: theme.dialogContentBackground
        border.color: theme.dialogBorderColor
        border.width: 1
        radius: theme.radiusSmall

        ScrollView {
            anchors.fill: parent
            anchors.margins: theme.spacingLarge
            contentWidth: availableWidth

            ColumnLayout {
                width: parent.width
                spacing: theme.spacingNormal

                Label { text: "Oracle 连接配置"; font.bold: true; font.pixelSize: theme.fontSizeTitle; color: theme.textPrimary; Layout.alignment: Qt.AlignHCenter }

                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    rowSpacing: theme.spacingNormal
                    columnSpacing: theme.spacingNormal

                    Label { text: "连接名称"; color: theme.textPrimary; font.pixelSize: theme.fontSizeNormal; Layout.preferredWidth: theme.formLabelWidth }
                    TextField { id: nameField; Layout.fillWidth: true; Layout.preferredWidth: theme.formInputWidth; text: root.conn ? root.conn.name : ""; placeholderText: "例如：本地Oracle"; onTextChanged: if (root.conn) root.conn.name = text; background: Rectangle { color: parent.activeFocus ? theme.backgroundColor : theme.inputFieldBackground; border.color: parent.activeFocus ? theme.inputFieldActiveBorder : theme.inputFieldBorder; border.width: 1; radius: theme.radiusSmall } }

                    Label { text: "主机地址"; color: theme.textPrimary; font.pixelSize: theme.fontSizeNormal; Layout.preferredWidth: theme.formLabelWidth }
                    TextField { Layout.fillWidth: true; Layout.preferredWidth: theme.formInputWidth; text: root.conn ? root.conn.host : ""; placeholderText: "localhost"; onTextChanged: if (root.conn) root.conn.host = text; background: Rectangle { color: parent.activeFocus ? theme.backgroundColor : theme.inputFieldBackground; border.color: parent.activeFocus ? theme.inputFieldActiveBorder : theme.inputFieldBorder; border.width: 1; radius: theme.radiusSmall } }

                    Label { text: "端口"; color: theme.textPrimary; font.pixelSize: theme.fontSizeNormal; Layout.preferredWidth: theme.formLabelWidth }
                    SpinBox { Layout.fillWidth: true; Layout.preferredWidth: theme.formInputWidth; from: 1; to: 65535; value: root.conn ? (root.conn.port || 1521) : 1521; onValueChanged: if (root.conn) root.conn.port = value; background: Rectangle { color: parent.activeFocus ? theme.backgroundColor : theme.inputFieldBackground; border.color: parent.activeFocus ? theme.inputFieldActiveBorder : theme.inputFieldBorder; border.width: 1; radius: theme.radiusSmall } }

                    Label { text: "用户名"; color: theme.textPrimary; font.pixelSize: theme.fontSizeNormal; Layout.preferredWidth: theme.formLabelWidth }
                    TextField { Layout.fillWidth: true; Layout.preferredWidth: theme.formInputWidth; text: root.conn ? root.conn.user : ""; onTextChanged: if (root.conn) root.conn.user = text; background: Rectangle { color: parent.activeFocus ? theme.backgroundColor : theme.inputFieldBackground; border.color: parent.activeFocus ? theme.inputFieldActiveBorder : theme.inputFieldBorder; border.width: 1; radius: theme.radiusSmall } }

                    Label { text: "密码"; color: theme.textPrimary; font.pixelSize: theme.fontSizeNormal; Layout.preferredWidth: theme.formLabelWidth }
                    TextField { Layout.fillWidth: true; Layout.preferredWidth: theme.formInputWidth; text: root.conn ? root.conn.password : ""; echoMode: TextInput.Password; onTextChanged: if (root.conn) root.conn.password = text; background: Rectangle { color: parent.activeFocus ? theme.backgroundColor : theme.inputFieldBackground; border.color: parent.activeFocus ? theme.inputFieldActiveBorder : theme.inputFieldBorder; border.width: 1; radius: theme.radiusSmall } }

                    Label { text: "服务名/SID"; color: theme.textPrimary; font.pixelSize: theme.fontSizeNormal; Layout.preferredWidth: theme.formLabelWidth }
                    TextField { Layout.fillWidth: true; Layout.preferredWidth: theme.formInputWidth; text: root.conn ? root.conn.database : ""; placeholderText: "XE / ORCL 等"; onTextChanged: if (root.conn) root.conn.database = text; background: Rectangle { color: parent.activeFocus ? theme.backgroundColor : theme.inputFieldBackground; border.color: parent.activeFocus ? theme.inputFieldActiveBorder : theme.inputFieldBorder; border.width: 1; radius: theme.radiusSmall } }
                }
            }
        }
    }

    onVisibleChanged: if (visible && nameField) nameField.forceActiveFocus()
}
