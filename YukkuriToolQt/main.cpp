#include <QApplication>
#include <QTranslator>
#include <QLibraryInfo>

#include <QDesktopWidget>
#include <QFontDatabase>

#include "mainwindow.h"

// macOSではノンネイティブなウィンドウがちゃんと表示されない
// ようなので、仕方が無いのでマクロでオン・オフしてあげる。
#if !defined(Q_OS_MAC) && 0
#define NON_NATIVE_WINDOW_FRAME
#endif

int main(int argc, char *argv[])
{
  QApplication a(argc, argv);

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
  mainWindow.setWindowFlags(mainWindow.windowFlags() | Qt::WindowStaysOnTopHint | Qt::Tool);
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
