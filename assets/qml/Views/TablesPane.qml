// 表列表页（QML）
// - 接收 connection 对象（id/name/...）
// - 通过 backend.listTables 查询并显示表名
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    property var connection: null

    // 主体布局：标题 + 表名列表
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        Label { text: (connection ? `表 - ${connection.name || connection.id}` : "表"); padding: 8; font.bold: true }
        ListView {
            id: tableList
            Layout.fillWidth: true
            Layout.fillHeight: true
            model: tables
            delegate: ItemDelegate { width: ListView.view.width; text: modelData }
            ScrollBar.vertical: ScrollBar {}
        }
    }

    // 表数据数组
    property var tables: []
    Component.onCompleted: reload()
    onConnectionChanged: reload()

    // 向后端请求表名列表
    function reload() {
        tables = []
        if (!connection) return
        const res = backend.listTables(connection)
        if (res.ok) tables = res.tables
        else console.warn("listTables error:", res.error)
    }
}
