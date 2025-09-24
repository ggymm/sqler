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

find_package(Qt6 6.2 COMPONENTS Widgets REQUIRED)

if(COMMAND qt_standard_project_setup)
    qt_standard_project_setup()
endif()

qt_add_executable(SimpleQtApp
    MANUAL_FINALIZATION
    src/main.cpp
)
set_target_properties(SimpleQtApp PROPERTIES MACOSX_BUNDLE TRUE)
target_link_libraries(SimpleQtApp PRIVATE Qt6::Widgets)

if(COMMAND qt_finalize_executable)
    qt_finalize_executable(SimpleQtApp)
endif()

if(APPLE)
    install(TARGETS SimpleQtApp BUNDLE DESTINATION .)
elseif(WIN32)
    install(TARGETS SimpleQtApp RUNTIME DESTINATION .)
else()
    install(TARGETS SimpleQtApp RUNTIME DESTINATION bin)
endif()

if(COMMAND qt_generate_deploy_app_script)
    qt_generate_deploy_app_script(TARGET SimpleQtApp OUTPUT_SCRIPT deploy_script)
    install(SCRIPT ${deploy_script})
endif()

add_custom_target(deploy
    COMMAND ${CMAKE_COMMAND} --install "${CMAKE_BINARY_DIR}" --prefix "${CMAKE_BINARY_DIR}/install"
    DEPENDS SimpleQtApp
)
