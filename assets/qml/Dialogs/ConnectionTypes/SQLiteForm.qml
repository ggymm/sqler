// SQLite 连接配置页面（选择数据库文件路径）
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

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: theme.spacingLarge
            spacing: theme.spacingNormal

            Label { text: "SQLite 连接配置"; font.bold: true; font.pixelSize: theme.fontSizeTitle; color: theme.textPrimary; Layout.alignment: Qt.AlignHCenter }

            GridLayout {
                Layout.fillWidth: true
                columns: 2
                rowSpacing: theme.spacingNormal
                columnSpacing: theme.spacingNormal

                Label { text: "连接名称"; color: theme.textPrimary; font.pixelSize: theme.fontSizeNormal; Layout.preferredWidth: theme.formLabelWidth }
                TextField { id: nameField; Layout.fillWidth: true; Layout.preferredWidth: theme.formInputWidth; text: root.conn ? root.conn.name : ""; placeholderText: "例如：本地SQLite"; onTextChanged: if (root.conn) root.conn.name = text; color: theme.inputTextColor; placeholderTextColor: theme.inputPlaceholderColor; selectionColor: theme.inputSelectionColor; selectedTextColor: theme.inputSelectedTextColor; background: Rectangle { color: parent.activeFocus ? theme.backgroundColor : theme.inputFieldBackground; border.color: parent.activeFocus ? theme.inputFieldActiveBorder : theme.inputFieldBorder; border.width: 1; radius: theme.radiusSmall } }

                Label { text: "数据库文件路径"; color: theme.textPrimary; font.pixelSize: theme.fontSizeNormal; Layout.preferredWidth: theme.formLabelWidth }
                TextField { id: pathField; Layout.fillWidth: true; Layout.preferredWidth: theme.formInputWidth; text: root.conn ? root.conn.database : ""; placeholderText: "输入 .db/.sqlite 文件路径"; onTextChanged: if (root.conn) root.conn.database = text; color: theme.inputTextColor; placeholderTextColor: theme.inputPlaceholderColor; selectionColor: theme.inputSelectionColor; selectedTextColor: theme.inputSelectedTextColor; background: Rectangle { color: parent.activeFocus ? theme.backgroundColor : theme.inputFieldBackground; border.color: parent.activeFocus ? theme.inputFieldActiveBorder : theme.inputFieldBorder; border.width: 1; radius: theme.radiusSmall } }
            }

            Label { text: "提示：暂未提供文件选择对话框，请直接输入文件绝对路径。"; color: theme.textSecondary; font.pixelSize: theme.fontSizeSmall }
        }
    }

    onVisibleChanged: if (visible && nameField) nameField.forceActiveFocus()
}
