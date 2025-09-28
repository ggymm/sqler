cmake_minimum_required(VERSION 3.16)

project(SimpleQtApp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTORCC ON)

find_package(Qt6 6.2 COMPONENTS Core Widgets REQUIRED)

qt_add_executable(sqler
    MANUAL_FINALIZATION
    src/main.cpp
    src/MainWindow.cpp src/MainWindow.h
    src/TopMenuBar.cpp src/TopMenuBar.h
    src/ConnectionPanel.cpp src/ConnectionPanel.h
    src/MainContent.cpp src/MainContent.h
    src/dialogs/NewConnectionDialog.cpp src/dialogs/NewConnectionDialog.h
    src/dialogs/DatabaseTypeDialog.cpp src/dialogs/DatabaseTypeDialog.h
    src/dialogs/ConnectionFormBase.cpp src/dialogs/ConnectionFormBase.h
    src/dialogs/MySQLConnectionForm.cpp src/dialogs/MySQLConnectionForm.h
    src/dialogs/PostgreSQLConnectionForm.cpp src/dialogs/PostgreSQLConnectionForm.h
    src/dialogs/SQLiteConnectionForm.cpp src/dialogs/SQLiteConnectionForm.h
    src/dialogs/MongoDBConnectionForm.cpp src/dialogs/MongoDBConnectionForm.h
    src/dialogs/RedisConnectionForm.cpp src/dialogs/RedisConnectionForm.h
    src/components/GStyle.h
    src/components/GPushButton.cpp src/components/GPushButton.h
    src/components/GLineEdit.cpp src/components/GLineEdit.h
    src/components/GSpinBox.cpp src/components/GSpinBox.h
    src/components/GLabel.cpp src/components/GLabel.h
    src/components/GScrollArea.cpp src/components/GScrollArea.h
    src/components/GDialog.cpp src/components/GDialog.h
    src/components/GListWidget.cpp src/components/GListWidget.h
    src/components/GSeparator.cpp src/components/GSeparator.h
    src/components/GConnectionItem.cpp src/components/GConnectionItem.h
    src/components/GTheme.cpp src/components/GTheme.h
)

qt_add_resources(sqler "assets"
    PREFIX "/"
    FILES
        assets/icons/new-conn.svg
        assets/icons/new-query.svg
        assets/icons/table.svg
        assets/icons/query.svg
        assets/icons/function.svg
        assets/icons/user.svg
        assets/icons/db/mysql.svg
        assets/icons/db/postgresql.svg
        assets/icons/db/sqlite.svg
        assets/icons/db/mongodb.svg
        assets/icons/db/redis.svg
        assets/icons/db/oracle.svg
        assets/icons/db/sqlserver.svg
)

target_link_libraries(sqler PRIVATE Qt6::Core Qt6::Widgets)
set_target_properties(sqler PROPERTIES MACOSX_BUNDLE TRUE)
qt_finalize_executable(sqler)
