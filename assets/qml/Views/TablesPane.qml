// 表列表页（QML）
// - 接收 connection 对象（id/name/...）
// - 通过 backendRef.listTables 查询并显示表名
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root

    property var connection: null
    property var backendRef: null

    // 表数据数组
    property var tables: []

    // 向后端请求表名列表
    function reload(): void {
        tables = [];
        if (!connection || !backendRef)
            return;
        const res = backendRef.listTables(connection);
        if (res.ok)
            tables = res.tables;
        else
            console.warn("listTables error:", res.error);
    }

    Component.onCompleted: reload()
    onConnectionChanged: reload()

    // 主体布局：标题 + 表名列表
    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Label {
            font.bold: true
            padding: 8
            text: (root.connection ? `表 - ${root.connection.name || root.connection.id}` : "表")
        }

        ListView {
            id: tableList

            Layout.fillHeight: true
            Layout.fillWidth: true
            model: root.tables

            ScrollBar.vertical: ScrollBar {
            }
            delegate: ItemDelegate {
                text: modelData
                width: ListView.view.width
            }
        }
    }
}
