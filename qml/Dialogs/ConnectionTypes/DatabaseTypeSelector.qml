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
                    { label: "MySQL", type: "mysql", description: "关系型数据库管理系统", icon: "qrc:/assets/icons/db/mysql.svg" },
                    { label: "Redis", type: "redis", description: "内存数据结构存储", icon: "qrc:/assets/icons/db/redis.svg" }
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
                            width: 48
                            height: 48
                            source: modelData.icon || ""
                            fillMode: Image.PreserveAspectFit
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
                        }

                        Label {
                            text: ">"
                            color: theme.textSecondary
                            font.pixelSize: theme.fontSizeLarge
                        }
                    }

                    onClicked: {
                        root.typeSelected(modelData.type)
                    }
                }
            }
        }
    }
}
