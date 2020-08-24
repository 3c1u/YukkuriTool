#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QMimeData>

#include "../riamify/riamify.h"

QT_BEGIN_NAMESPACE
namespace Ui { class MainWindow; }
QT_END_NAMESPACE

class MainWindow : public QMainWindow
{
  Q_OBJECT

public:
  MainWindow(QWidget *parent = nullptr);
  ~MainWindow();

public slots:
  void windowActivated();

  void generateVoice();
  void resetField();
  void speakText();
  void setClipboardPasteEnabled(bool);
  void setClipboardCopyEnabled(bool);
  void openSettings();
  void managePresets();

  void setPreset(int id);
  void setSpeed(int speed);

  void generateYomi();

  void resetDisplay();

  virtual void mousePressEvent(QMouseEvent *event);
private:
  Ui::MainWindow *ui;

  bool m_windowStatus;
  bool m_msgBoxInvoked;
  RiamifyApp *m_app;

  QMimeData *m_data = nullptr;

  void setup();

  void invokeErrorMessage(QString const& message);
};
#endif // MAINWINDOW_H
