// 数据库类型选择页面
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root

    property var theme
    property var conn

    signal typeSelected(string type)

    Rectangle {
        anchors.fill: parent
        color: theme.dialogContentBackground
        border.color: theme.dialogBorderColor
        border.width: 1
        radius: theme.radiusSmall

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: theme.spacingLarge
            spacing: theme.spacingLarge

            Label {
                text: "选择数据库类型"
                font.bold: true
                font.pixelSize: theme.fontSizeTitle
                color: theme.textPrimary
                Layout.alignment: Qt.AlignHCenter
            }

            ListView {
                id: listView
                Layout.fillWidth: true
                Layout.fillHeight: true
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
                spacing: theme.spacingNormal

                delegate: ItemDelegate {
                    width: listView.width
                    height: 80

                    background: Rectangle {
                        color: parent.hovered ? theme.selectorItemHoverBackground : theme.selectorItemBackground
                        border.color: parent.hovered ? theme.primaryColor : theme.selectorItemBorder
                        border.width: 1
                        radius: theme.radiusNormal
                    }

                    contentItem: RowLayout {
                        spacing: theme.spacingNormal

                        Image {
                            // Use Layout attached properties inside RowLayout
                            Layout.preferredWidth: theme.iconSizeLarge
                            Layout.preferredHeight: theme.iconSizeLarge
                            Layout.alignment: Qt.AlignVCenter
                            source: modelData.icon || ""
                            fillMode: Image.PreserveAspectFit
                            smooth: true
                            // Ensure the decoder scales image efficiently
                            sourceSize.width: theme.iconSizeLarge
                            sourceSize.height: theme.iconSizeLarge
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: theme.spacingTiny

                            Label {
                                text: modelData.label || ""
                                color: theme.textPrimary
                                font.bold: true
                                font.pixelSize: theme.fontSizeNormal
                            }

                            Label {
                                text: modelData.description || ""
                                color: theme.textSecondary
                                font.pixelSize: theme.fontSizeSmall
                            }
                            // Tag when unsupported
                            Rectangle {
                                visible: !modelData.supported
                                radius: theme.radiusSmall
                                color: theme.selectedColor
                                border.color: theme.borderColor
                                border.width: 1
                                anchors.leftMargin: theme.spacingSmall
                                width: implicitWidth
                                height: implicitHeight
                                Row {
                                    spacing: theme.spacingTiny
                                    anchors.margins: theme.spacingTiny
                                    anchors.fill: parent
                                    Label {
                                        text: "未支持"
                                        color: theme.textPrimary
                                        font.pixelSize: theme.fontSizeSmall
                                    }
                                }
                            }
                        }

                        Label {
                            text: ">"
                            color: theme.textSecondary
                            font.pixelSize: theme.fontSizeLarge
                        }
                    }

                    onClicked: root.typeSelected(modelData.type)
                }
            }
        }
    }
}
