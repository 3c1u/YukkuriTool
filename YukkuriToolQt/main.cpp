#include <QApplication>
#include <QTranslator>
#include <QLibraryInfo>

#include <QDesktopWidget>
#include <QFontDatabase>

#include <QStyleFactory>

#include "mainwindow.h"

// macOSではノンネイティブなウィンドウがちゃんと表示されない
// ようなので、仕方が無いのでマクロでオン・オフしてあげる。
#if !defined(Q_OS_MAC) && 0
#define NON_NATIVE_WINDOW_FRAME
#endif

int main(int argc, char *argv[])
{
  QApplication a(argc, argv);
  a.setStyle(QStyleFactory::create("Fusion"));

  QPalette darkPalette;
  darkPalette.setColor(QPalette::Window, QColor(0x30, 0x32, 0x33));
  darkPalette.setColor(QPalette::WindowText, Qt::white);
  darkPalette.setColor(QPalette::Base, QColor(25,25,25));
  darkPalette.setColor(QPalette::AlternateBase, QColor(153,153,153));
  darkPalette.setColor(QPalette::ToolTipBase, Qt::white);
  darkPalette.setColor(QPalette::ToolTipText, Qt::white);
  darkPalette.setColor(QPalette::Text, Qt::white);
  darkPalette.setColor(QPalette::Light, QColor(0xe0, 0xef, 0xe6));
  darkPalette.setColor(QPalette::Button, QColor(0x40, 0x43, 0x46));
  darkPalette.setColor(QPalette::ButtonText, Qt::white);
  darkPalette.setColor(QPalette::BrightText, Qt::red);
  darkPalette.setColor(QPalette::Link, QColor(0x56, 0xc8, 0xd8));

  darkPalette.setColor(QPalette::Highlight, QColor(0x56, 0xe2, 0xd8));
  darkPalette.setColor(QPalette::HighlightedText, Qt::black);

  a.setPalette(darkPalette);

  a.setStyleSheet("QToolTip { color: #ffffff; background-color: #2a82da; border: 1px solid white; }");

  QTranslator qt_translator;
  qt_translator.load("qtbase_" + QLocale::system().name(),
                     QLibraryInfo::location(QLibraryInfo::TranslationsPath));

  QTranslator app_translator;
  app_translator.load("YukkuriToolQt_" + QLocale::system().name(), QLatin1String(":/i18n"));

  a.installTranslator(&qt_translator);
  a.installTranslator(&app_translator);

#if defined(Q_OS_MAC)
    //  適切な代替フォントの挿入
    QFont::insertSubstitution(".AppleSystemUIFont", "Hiragino Sans");
#endif

#if defined(NON_NATIVE_WINDOW_FRAME)

  FramelessWindow framelessWindow;
  MainWindow *mainWindow = new MainWindow();

#if defined(Q_OS_MAC)
  framelessWindow.setWindowFlags(framelessWindow.windowFlags() | Qt::WindowStaysOnTopHint);
#else
  framelessWindow.setWindowFlags(framelessWindow.windowFlags() | Qt::WindowStaysOnTopHint);
  auto font = QFont("Yu Gothic UI", 9);
  a.setFont(font);
#endif // defined(Q_OS_MAC)

  framelessWindow.setWindowTitle(QObject::tr("YukkuriTool"));
  framelessWindow.setAttribute(Qt::WA_QuitOnClose);

  framelessWindow.move(QApplication::desktop()->screen()->rect().bottomRight()
                  - framelessWindow.rect().bottomRight()
                  - QPoint(15, 70));

  framelessWindow.setContent(mainWindow);
  framelessWindow.show();

#else
  MainWindow mainWindow;
#if defined(Q_OS_MAC)
  mainWindow.setWindowFlags(mainWindow.windowFlags() | Qt::WindowStaysOnTopHint);
#else
  mainWindow.setWindowFlags(mainWindow.windowFlags() | Qt::WindowStaysOnTopHint);
  a.setFont(QFont("Yu Gothic UI", 9));
#endif
  mainWindow.setWindowTitle(QObject::tr("YukkuriTool"));
  mainWindow.setAttribute(Qt::WA_QuitOnClose);

  mainWindow.move(QApplication::desktop()->screen()->rect().bottomRight()
                  - QPoint(mainWindow.rect().width(), mainWindow.rect().height())
                       - QPoint(15, 90));

  mainWindow.show();
#endif // NON_NATIVE_WINDOW_FRAME

  return a.exec();
}
