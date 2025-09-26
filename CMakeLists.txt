cmake_minimum_required(VERSION 3.16)

project(sqler LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(QT_ROOT "" CACHE PATH "Qt 6 installation prefix (optional)")
if(QT_ROOT)
    list(PREPEND CMAKE_PREFIX_PATH "${QT_ROOT}")
endif()

find_package(Qt6 6.2 COMPONENTS Quick QuickControls2 Qml Sql Network REQUIRED)

if(COMMAND qt_standard_project_setup)
    qt_standard_project_setup()
endif()

add_executable(sqler
    src/main.cpp
    src/core/Config.cpp
    src/core/ConnectionStore.cpp
    src/db/DbUtil.cpp
    src/db/Drivers.cpp
    src/qml/Backend.cpp
    res/resources.qrc
)
set_target_properties(sqler PROPERTIES MACOSX_BUNDLE TRUE)
target_include_directories(sqler PRIVATE src)
target_link_libraries(sqler PRIVATE Qt6::Quick Qt6::QuickControls2 Qt6::Qml Qt6::Sql Qt6::Network)

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
    DEPENDS sqler
)
