// 新建/编辑连接对话框（QML 子窗口模式）
// 使用模块化组件：DatabaseTypeSelector, 各数据库表单
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick.Window 2.15
import "ConnectionTypes" as ConnectionTypes

Window {
    id: root

    property var conn: ({
                            id: "",
                            name: "",
                            type: "",
                            host: "localhost",
                            port: 3306,
                            user: "",
                            password: "",
                            database: ""
                        })
    property int currentStep: 0 // 0: type selection, 1: form
    // 作为子窗口，设置父窗口用于窗口置顶与模态
    property var parentWindow: null
    // 当前已实现类型
    property var supportedTypes: ["mysql", "redis"]
    property var theme
    // Qualified backend reference injected by Main
    property var backendRef: null

    function closeDialog(): void {
        root.close();
    }

    function focusFirstField(): void {
        try {
            if (conn.type === 'mysql' && mysqlForm.focusFirst)
                mysqlForm.focusFirst();
            else if (conn.type === 'redis' && redisForm.focusFirst)
                redisForm.focusFirst();
            else if (conn.type === 'postgresql' && pgForm.focusFirst)
                pgForm.focusFirst();
            else if (conn.type === 'sqlserver' && mssqlForm.focusFirst)
                mssqlForm.focusFirst();
            else if (conn.type === 'sqlite' && sqliteForm.focusFirst)
                sqliteForm.focusFirst();
            else if (conn.type === 'mongodb' && mongoForm.focusFirst)
                mongoForm.focusFirst();
            else if (conn.type === 'oracle' && oracleForm.focusFirst)
                oracleForm.focusFirst();
        } catch (e) {
            console.warn('focusFirstField error', e);
        }
    }

    function handleCancel(): void {
        closeDialog();
    }

    function handleNext(): void {
        if (currentStep === 0) {
            // Need to select type first
            return;
        }
        // Save connection
        if (!backendRef) { closeDialog(); return; }
        const id = backendRef.saveConnection(conn);
        if (id && id.length > 0)
            backendRef.refreshConnections();
        closeDialog();
    }

    function handlePrevious(): void {
        if (currentStep > 0) {
            currentStep--;
        }
    }

    function handleTest(): void {
        if (!backendRef) return;
        const res = backendRef.testConnection(conn);
        if (res.ok)
            infoDialog.show("连接成功");
        else
            infoDialog.show("测试连接失败: " + (res.error || ""));
    }

    function onTypeSelected(type: string): void {
        console.log("Type selected:", type);
        conn.type = type;
        console.log("Connection type set to:", conn.type);
        // Set default values based on type
        if (type === 'mysql') {
            conn.port = 3306;
            conn.user = 'root';
            conn.database = '';
        } else if (type === 'redis') {
            conn.port = 6379;
            conn.user = '';
            conn.database = '0';
        } else if (type === 'postgresql') {
            conn.port = 5432;
            conn.user = 'postgres';
            conn.database = '';
        } else if (type === 'sqlserver') {
            conn.port = 1433;
            conn.user = 'sa';
            conn.database = '';
        } else if (type === 'sqlite') {
            conn.port = 0;
            conn.user = '';
            conn.database = '';
        } else if (type === 'mongodb') {
            conn.port = 27017;
            conn.user = '';
            conn.database = '';
        } else if (type === 'oracle') {
            conn.port = 1521;
            conn.user = '';
            conn.database = '';
        }
        currentStep = 1;
        console.log("Current step set to:", currentStep);
        Qt.callLater(focusFirstField);
    }

    function openForEdit(c: var): void {
        conn = Object.assign({}, c);
        if (!conn.type)
            conn.type = "mysql";
        currentStep = 1;
        root.show();
        root.raise();
        root.requestActivate();
    }

    function openForNew(): void {
        conn = {
            id: "",
            name: "",
            type: "",
            host: "localhost",
            port: 3306,
            user: "",
            password: "",
            database: ""
        };
        currentStep = 0;
        root.show();
        root.raise();
        root.requestActivate();
    }

    color: theme.dialogBackgroundColor
    // 显示系统窗口控制按钮（关闭/最小化/最大化）
    flags: Qt.Window | Qt.WindowTitleHint | Qt.WindowCloseButtonHint | Qt.WindowMinimizeButtonHint
           | Qt.WindowMaximizeButtonHint | Qt.WindowSystemMenuHint
    height: 550
    modality: Qt.ApplicationModal
    title: currentStep === 0 ? "新建连接" : (conn.type.toUpperCase() + " 连接配置")
    visible: false
    width: theme.dialogWidth

    onParentWindowChanged: if (parentWindow)
                               root.transientParent = parentWindow

    // Main content area
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: root.theme.spacingNormal
        spacing: 0

        // Content area
        Item {
            Layout.fillHeight: true
            Layout.fillWidth: true

            // Type selection page
            ConnectionTypes.DatabaseTypeSelector {
                id: typeSelector

                anchors.fill: parent
                conn: root.conn
                theme: root.theme
                visible: root.currentStep === 0

                onTypeSelected: function (type) {
                    root.onTypeSelected(type);
                }
            }

            // MySQL form
            ConnectionTypes.MySQLForm {
                id: mysqlForm

                anchors.fill: parent
                conn: root.conn
                theme: root.theme
                visible: root.currentStep === 1 && conn.type === "mysql"
            }

            // Redis form
            ConnectionTypes.RedisForm {
                id: redisForm

                anchors.fill: parent
                conn: root.conn
                theme: root.theme
                visible: root.currentStep === 1 && conn.type === "redis"
            }

            // PostgreSQL form
            ConnectionTypes.PostgreSQLForm {
                id: pgForm

                anchors.fill: parent
                conn: root.conn
                theme: root.theme
                visible: root.currentStep === 1 && conn.type === "postgresql"
            }

            // SQL Server form
            ConnectionTypes.SQLServerForm {
                id: mssqlForm

                anchors.fill: parent
                conn: root.conn
                theme: root.theme
                visible: root.currentStep === 1 && conn.type === "sqlserver"
            }

            // SQLite form
            ConnectionTypes.SQLiteForm {
                id: sqliteForm

                anchors.fill: parent
                conn: root.conn
                theme: root.theme
                visible: root.currentStep === 1 && conn.type === "sqlite"
            }

            // MongoDB form
            ConnectionTypes.MongoDBForm {
                id: mongoForm

                anchors.fill: parent
                conn: root.conn
                theme: root.theme
                visible: root.currentStep === 1 && conn.type === "mongodb"
            }

            // Oracle form
            ConnectionTypes.OracleForm {
                id: oracleForm

                anchors.fill: parent
                conn: root.conn
                theme: root.theme
                visible: root.currentStep === 1 && conn.type === "oracle"
            }

            // Unsupported placeholder
            Item {
                id: unsupported

                anchors.fill: parent
                visible: root.currentStep === 1 && (root.conn.type !== "mysql" && root.conn.type
                                                    !== "redis" && root.conn.type !== "postgresql"
                                                    && root.conn.type !== "sqlserver"
                                                    && root.conn.type !== "sqlite"
                                                    && root.conn.type !== "mongodb"
                                                    && root.conn.type !== "oracle")

                Rectangle {
                    anchors.centerIn: parent
                    border.color: root.theme.dialogBorderColor
                    border.width: 1
                    color: root.theme.dialogContentBackground
                    height: 160
                    radius: root.theme.radiusNormal
                    width: parent.width * 0.8

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: root.theme.spacingLarge
                        spacing: root.theme.spacingSmall

                        Label {
                            Layout.alignment: Qt.AlignHCenter
                            color: root.theme.textPrimary
                            font.bold: true
                            font.pixelSize: root.theme.fontSizeTitle
                            text: (root.conn.type || '').toUpperCase() + " 暂未支持"
                        }

                        Label {
                            Layout.alignment: Qt.AlignHCenter
                            color: root.theme.textSecondary
                            font.pixelSize: root.theme.fontSizeNormal
                            horizontalAlignment: Text.AlignHCenter
                            text: "暂不支持该数据库类型的配置与连接，敬请期待。"
                            wrapMode: Text.WordWrap
                        }
                    }
                }
            }
        }

        // Button area
        Rectangle {
            Layout.fillWidth: true
            border.color: root.theme.dialogBorderColor
            border.width: 1
            color: root.theme.dialogContentBackground
            Layout.preferredHeight: 60

            RowLayout {
                anchors.fill: parent
                anchors.margins: root.theme.spacingNormal
                spacing: root.theme.spacingNormal

                Button {
                    Layout.preferredWidth: 100
                    enabled: root.currentStep === 1 && root.supportedTypes.indexOf(root.conn.type) !==
                             -1
                    text: "测试连接"

                    background: Rectangle {
                        border.color: root.theme.borderColor
                        border.width: 1
                        color: parent.enabled ? (parent.hovered ? root.theme.hoverColor :
                                                                  "transparent") :
                                                root.theme.backgroundColor
                        radius: root.theme.radiusSmall
                    }
                    contentItem: Text {
                        color: parent.enabled ? root.theme.textPrimary : root.theme.textHint
                        font.pixelSize: root.theme.fontSizeNormal
                        horizontalAlignment: Text.AlignHCenter
                        text: parent.text
                        verticalAlignment: Text.AlignVCenter
                    }

                    onClicked: handleTest()
                }

                Item {
                    Layout.fillWidth: true
                } // Spacer



                Button {
                    Layout.preferredWidth: 80
                    text: "上一步"
                    visible: root.currentStep > 0

                    background: Rectangle {
                        border.color: root.theme.borderColor
                        border.width: 1
                        color: parent.hovered ? root.theme.hoverColor : "transparent"
                        radius: root.theme.radiusSmall
                    }
                    contentItem: Text {
                        color: root.theme.textPrimary
                        font.pixelSize: root.theme.fontSizeNormal
                        horizontalAlignment: Text.AlignHCenter
                        text: parent.text
                        verticalAlignment: Text.AlignVCenter
                    }

                    onClicked: handlePrevious()
                }

                Button {
                    Layout.preferredWidth: 80
                    enabled: root.currentStep === 0 || root.currentStep === 1
                    text: root.currentStep === 0 ? "下一步" : "确定"

                    background: Rectangle {
                        color: parent.hovered ? root.theme.primaryColorLight :
                                                root.theme.primaryColor
                        radius: root.theme.radiusSmall
                    }
                    contentItem: Text {
                        color: "white"
                        font.bold: true
                        font.pixelSize: root.theme.fontSizeNormal
                        horizontalAlignment: Text.AlignHCenter
                        text: parent.text
                        verticalAlignment: Text.AlignVCenter
                    }

                    onClicked: handleNext()
                }

                Button {
                    Layout.preferredWidth: 80
                    text: "取消"

                    background: Rectangle {
                        border.color: root.theme.borderColor
                        border.width: 1
                        color: parent.hovered ? root.theme.hoverColor : "transparent"
                        radius: root.theme.radiusSmall
                    }
                    contentItem: Text {
                        color: root.theme.textPrimary
                        font.pixelSize: root.theme.fontSizeNormal
                        horizontalAlignment: Text.AlignHCenter
                        text: parent.text
                        verticalAlignment: Text.AlignVCenter
                    }

                    onClicked: handleCancel()
                }
            }
        }
    }

    // Info dialog
    Dialog {
        id: infoDialog

        property alias text: msgLabel.text

        function show(message) {
            text = message;
            open();
        }

        modal: true
        standardButtons: Dialog.Ok
        title: "信息"

        background: Rectangle {
            border.color: root.theme.borderColor
            border.width: root.theme.dialogBorderWidth
            color: root.theme.surfaceColor
            radius: root.theme.radiusNormal
        }
        contentItem: Label {
            id: msgLabel

            color: root.theme.textPrimary
            padding: root.theme.spacingNormal
        }
    }
}
