pragma ComponentBehavior: Bound
// 数据库类型选择页面
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root

    property var conn
    property var theme

    signal typeSelected(string type)

    Rectangle {
        anchors.fill: parent
        border.color: root.theme.dialogBorderColor
        border.width: 1
        color: root.theme.dialogContentBackground
        radius: root.theme.radiusSmall

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: root.theme.spacingLarge
            spacing: root.theme.spacingLarge

            Label {
                Layout.alignment: Qt.AlignHCenter
                color: root.theme.textPrimary
                font.bold: true
                font.pixelSize: root.theme.fontSizeTitle
                text: "选择数据库类型"
            }

            ListView {
                id: listView

                Layout.fillHeight: true
                Layout.fillWidth: true
                model: [
                    {
                        label: "MySQL",
                        type: "mysql",
                        description: "关系型数据库",
                        icon: "qrc:/assets/icons/db/mysql.svg",
                        supported: true
                    },
                    {
                        label: "PostgreSQL",
                        type: "postgresql",
                        description: "关系型数据库",
                        icon: "qrc:/assets/icons/db/postgresql.svg",
                        supported: false
                    },
                    {
                        label: "SQL Server",
                        type: "sqlserver",
                        description: "关系型数据库",
                        icon: "qrc:/assets/icons/db/sqlserver.svg",
                        supported: false
                    },
                    {
                        label: "SQLite",
                        type: "sqlite",
                        description: "嵌入式数据库",
                        icon: "qrc:/assets/icons/db/sqlite.svg",
                        supported: false
                    },
                    {
                        label: "MongoDB",
                        type: "mongodb",
                        description: "文档数据库",
                        icon: "qrc:/assets/icons/db/mongodb.svg",
                        supported: false
                    },
                    {
                        label: "Oracle",
                        type: "oracle",
                        description: "关系型数据库",
                        icon: "qrc:/assets/icons/db/oracle.svg",
                        supported: false
                    },
                    {
                        label: "Redis",
                        type: "redis",
                        description: "内存数据结构存储",
                        icon: "qrc:/assets/icons/db/redis.svg",
                        supported: true
                    }
                ]
                spacing: root.theme.spacingNormal

                delegate: ItemDelegate {
                    id: cell
                    height: 80
                    width: listView.width

                    background: Rectangle {
                        border.color: cell.hovered ? root.theme.primaryColor : root.theme.selectorItemBorder
                        border.width: 1
                        color: cell.hovered ? root.theme.selectorItemHoverBackground :
                                              root.theme.selectorItemBackground
                        radius: root.theme.radiusNormal
                    }
                    contentItem: RowLayout {
                        spacing: root.theme.spacingNormal

                        Image {
                            Layout.alignment: Qt.AlignVCenter
                            Layout.preferredHeight: root.theme.iconSizeLarge
                            // Use Layout attached properties inside RowLayout
                            Layout.preferredWidth: root.theme.iconSizeLarge
                            fillMode: Image.PreserveAspectFit
                            smooth: true
                            source: modelData.icon || ""
                            sourceSize.height: root.theme.iconSizeLarge
                            // Ensure the decoder scales image efficiently
                            sourceSize.width: root.theme.iconSizeLarge
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: root.theme.spacingTiny

                            Label {
                                color: root.theme.textPrimary
                                font.bold: true
                                font.pixelSize: root.theme.fontSizeNormal
                                text: modelData.label || ""
                            }

                            Label {
                                color: root.theme.textSecondary
                                font.pixelSize: root.theme.fontSizeSmall
                                text: modelData.description || ""
                            }

                            // Tag when unsupported
                            Rectangle {
                                Layout.leftMargin: root.theme.spacingSmall
                                border.color: root.theme.borderColor
                                border.width: 1
                                color: root.theme.selectedColor
                                Layout.preferredHeight: implicitHeight
                                radius: root.theme.radiusSmall
                                visible: !modelData.supported
                                Layout.preferredWidth: implicitWidth

                                Row {
                                    anchors.fill: parent
                                    anchors.margins: root.theme.spacingTiny
                                    spacing: root.theme.spacingTiny

                                    Label {
                                        color: root.theme.textPrimary
                                        font.pixelSize: root.theme.fontSizeSmall
                                        text: "未支持"
                                    }
                                }
                            }
                        }

                        Label {
                            color: root.theme.textSecondary
                            font.pixelSize: root.theme.fontSizeLarge
                            text: ">"
                        }
                    }

                    onClicked: root.typeSelected(modelData.type)
                }
            }
        }
    }
}
