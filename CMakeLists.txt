cmake_minimum_required(VERSION 3.16)

project(SimpleQtApp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTORCC ON)

set(QT_ROOT "" CACHE PATH "Qt 6 installation prefix (optional)")
if(QT_ROOT)
    list(PREPEND CMAKE_PREFIX_PATH "${QT_ROOT}")
endif()

find_package(Qt6 6.2 COMPONENTS Core Widgets REQUIRED)

if(COMMAND qt_standard_project_setup)
    qt_standard_project_setup()
endif()

qt_add_executable(sqler
        MANUAL_FINALIZATION
        src/main.cpp
        src/MainWindow.cpp
        src/MainWindow.h
        src/TopMenuBar.cpp
        src/TopMenuBar.h
        src/ConnectionPanel.cpp
        src/ConnectionPanel.h
        src/MainContent.cpp
        src/MainContent.h
        src/dialogs/NewConnectionDialog.cpp
        src/dialogs/NewConnectionDialog.h
        src/dialogs/DatabaseTypeDialog.cpp
        src/dialogs/DatabaseTypeDialog.h
        src/dialogs/ConnectionFormBase.cpp
        src/dialogs/ConnectionFormBase.h
        src/dialogs/MySQLConnectionForm.cpp
        src/dialogs/MySQLConnectionForm.h
        src/dialogs/PostgreSQLConnectionForm.cpp
        src/dialogs/PostgreSQLConnectionForm.h
        src/dialogs/SQLiteConnectionForm.cpp
        src/dialogs/SQLiteConnectionForm.h
        src/dialogs/MongoDBConnectionForm.cpp
        src/dialogs/MongoDBConnectionForm.h
        src/dialogs/RedisConnectionForm.cpp
        src/dialogs/RedisConnectionForm.h
        src/components/ThemedButton.cpp
        src/components/ThemedButton.h
        src/components/ThemedScrollArea.cpp
        src/components/ThemedScrollArea.h
        src/components/ThemedDialog.cpp
        src/components/ThemedDialog.h
        src/components/ThemedFormWidget.cpp
        src/components/ThemedFormWidget.h
        src/Theme.cpp
        src/Theme.h
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
set_target_properties(sqler PROPERTIES MACOSX_BUNDLE TRUE)
target_link_libraries(sqler PRIVATE Qt6::Core Qt6::Widgets)

if(COMMAND qt_finalize_executable)
    qt_finalize_executable(sqler)
endif()

if(APPLE)
    install(TARGETS sqler BUNDLE DESTINATION .)
elseif(WIN32)
    install(TARGETS sqler RUNTIME DESTINATION .)
else()
    install(TARGETS sqler RUNTIME DESTINATION bin)
endif()

if(COMMAND qt_generate_deploy_app_script)
    qt_generate_deploy_app_script(TARGET sqler OUTPUT_SCRIPT deploy_script)
    install(SCRIPT ${deploy_script})
endif()

add_custom_target(deploy
        COMMAND ${CMAKE_COMMAND} --install "${CMAKE_BINARY_DIR}" --prefix "${CMAKE_BINARY_DIR}/install"
        DEPENDS SimpleQtApp
)
