cmake_minimum_required(VERSION 3.5)

project(YukkuriToolQt LANGUAGES CXX)

set(CMAKE_INCLUDE_CURRENT_DIR ON)

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt5 COMPONENTS Widgets LinguistTools REQUIRED)

set(TS_FILES YukkuriToolQt_ja_JP.ts YukkuriToolQt_en_US.ts)
qt5_create_translation(QM_FILES ${CMAKE_SOURCE_DIR} ${TS_FILES})
qt5_add_translation(QM ${TS_FILES})

set(lang_qrc "lang.qrc")
configure_file(${lang_qrc} ${lang_qrc} COPYONLY)

qt5_add_translation(YTQT_QM ${myapp_TRANSLATIONS})

qt5_add_resources(
    RCC_SOURCES
    assets.qrc
    ${CMAKE_CURRENT_BINARY_DIR}/${lang_qrc}
)

add_executable(YukkuriToolQt
  main.cpp
  mainwindow.cpp
  RiamifyFFI.cpp
  mainwindow.h
  # forms
  mainwindow.ui
  # resources
  ${RCC_SOURCES}
  ${QM}
)

target_compile_definitions(YukkuriToolQt PUBLIC UNICODE _UNICODE)

add_custom_target(
    YukkuriToolCore
    COMMAND cargo build --release --target i686-pc-windows-msvc
)

add_dependencies(YukkuriToolQt YukkuriToolCore)

add_custom_command(TARGET YukkuriToolQt
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "C:/Program Files (x86)/MeCab/bin/libmecab.dll"
        $<TARGET_FILE_DIR:YukkuriToolQt>)

target_link_directories(YukkuriToolQt PRIVATE ${CMAKE_SOURCE_DIR}/../target/i686-pc-windows-msvc/release)
target_link_directories(YukkuriToolQt PRIVATE "C:/Program Files (x86)/MeCab/sdk")

target_link_libraries(YukkuriToolQt PRIVATE
    Qt5::Widgets
    riamify
    libmecab
    WS2_32.lib
    User32.lib
    advapi32.lib
    userenv.lib
    ole32.lib
    winmm.lib
    dwmapi.lib
    uxtheme.lib
)
