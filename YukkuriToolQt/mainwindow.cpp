#include "mainwindow.h"
#include "ui_mainwindow.h"

#include "../riamify/riamify.h"

#include <QClipboard>
#include <QMimeData>
#include <QMessageBox>

#include <QDrag>


#ifdef Q_OS_WIN

#include <Windows.h>
#include <uxtheme.h>
#include <dwmapi.h>
#include <mmsystem.h>

#endif

// #define DEMO_MODE

extern "C" void ___SetDarkMode();

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
      , ui(new Ui::MainWindow)
{
#ifdef Q_OS_MAC
  ___SetDarkMode();
#endif

  ui->setupUi(this);

#ifdef Q_OS_WIN
  auto hwnd = HWND(winId());
  const BOOL is_dark_mode = true;
  assert(SetWindowTheme(hwnd, L"DarkMode_Explorer", nullptr) == S_OK);
  assert(DwmSetWindowAttribute(hwnd, 20, &is_dark_mode, sizeof(is_dark_mode)) == S_OK);
#endif

  connect(qApp, SIGNAL(focusChanged(QWidget*, QWidget*)),
          this, SLOT(windowActivated()));

  char *err = nullptr;

  m_app = RiamifyAppCreate(&err);

  if (err != nullptr) {
      QMessageBox::critical(this,
                            tr("Failed to initialize the application"),
                            QString::fromUtf8(err));
      RiamifyReleasePointer(err);
      exit(0);
  }

  // プリセット一覧を生成
  setup();

  ui->kevinView->setCursor(QCursor(Qt::SizeAllCursor));

  // macOS用のフォント設定をする
#if defined(Q_OS_MAC)
  ui->plainTextEdit->setFont(QFont("Hiragino Sans", -1, 57));
  ui->plainTextEdit_2->setFont(QFont("Hiragino Sans", -1, 57));
#endif
}

void MainWindow::setup() {
#if defined(DEMO_MODE)
  size_t presets_len = 1;
  ui->voicePresets->setEnabled(false);
#else
  auto presets_len = RiamifyAppGetPresetsLength(m_app);
#endif


#if defined(DEMO_MODE)
  ui->pushButton->setEnabled(false);
  ui->checkBox->setEnabled(false);
  ui->checkBox->setCheckState(Qt::Unchecked);
  ui->checkBox_2->setEnabled(false);
#endif

  ui->voicePresets->clear();

  for (size_t i = 0; i < presets_len; i++) {
    auto preset = RiamifyAppGetPresetName(m_app, static_cast<int>(i));
    auto presetName = QString::fromUtf8(reinterpret_cast<const char*>(preset.ptr),
                                        static_cast<int>(preset.len));

    ui->voicePresets->addItem(presetName);
  }

  RiamifyAppSetPreset(m_app, 0);
}

void MainWindow::setPreset(int id) {
  RiamifyAppSetPreset(m_app, id);
}

void MainWindow::setSpeed(int speed) {
  RiamifyAppSetSpeed(m_app, speed);
}

void MainWindow::windowActivated() {
  auto status = this->isActiveWindow();

  if (m_windowStatus == status) {
    return;
  }

  m_windowStatus = status;

  // 設定またはプリセットが変わっているときは更新する．
  if (RiamifyAppUpdatePresets(m_app)) {
    // プリセット一覧の更新
    setup();
  }

  if (status && ui->checkBox->isChecked()) {
    //
    if (m_msgBoxInvoked) {
      m_msgBoxInvoked = false;
      return;
    }

    // ウィンドウにフォーカスが移ったときに、クリップボードの内容を反映する
    auto clipboard = QApplication::clipboard();
    if (clipboard->mimeData()->hasText()
        && !clipboard->mimeData()->hasFormat("text/uri-list")) {
      auto text = clipboard->text();
      if (text == ui->plainTextEdit->toPlainText())
        return;

      auto cursor = ui->plainTextEdit->textCursor();

      cursor.beginEditBlock();

      cursor.select(QTextCursor::Document);
      cursor.removeSelectedText();

      cursor.insertText(text);
      cursor.clearSelection();

      cursor.endEditBlock();

      ui->plainTextEdit->setTextCursor(cursor);
    }
  } else if (ui->checkBox_2->isChecked()) {
    // フォーカスが外れたときにクリップボードにWAVファイルを読み込ませる
    this->generateVoice();

    if (m_data) {
        auto clipboard = QApplication::clipboard();
        clipboard->setMimeData(m_data);
        m_data = nullptr;
    }
  }
}

MainWindow::~MainWindow()
{
  delete ui;
  RiamifyAppRelease(m_app);
}

void MainWindow::generateYomi() {
  auto text = ui->plainTextEdit->toPlainText().toUtf8();

  auto riamified = RiamifyAppGetYomi(m_app, reinterpret_cast<const int8_t*>(text.data()));

  auto rQ = QString::fromUtf8(reinterpret_cast<const char*>(riamified));
  auto cursor = ui->plainTextEdit_2->textCursor();

  cursor.beginEditBlock();
  cursor.select(QTextCursor::Document);
  cursor.removeSelectedText();
  cursor.clearSelection();

  cursor.insertText(rQ);

  cursor.endEditBlock();

  ui->plainTextEdit_2->setTextCursor(cursor);

  RiamifyReleasePointer(riamified);
}

void MainWindow::mousePressEvent(QMouseEvent *event) {
  if (event->button() == Qt::LeftButton &&
      ui->kevinView->underMouse()) {
    this->generateVoice();

    if (m_data == nullptr)
      return;

    QDrag *drag = new QDrag(this);

    drag->setMimeData(m_data);
    m_data = nullptr;

    // ケビンの無効化
    // drag->setPixmap(QPixmap::fromImage(QImage(":/image/kevin.jpg")).scaled(24, 24));

    drag->exec(Qt::CopyAction | Qt::MoveAction, Qt::CopyAction);
  }
}

void MainWindow::generateVoice() {
  auto text = ui->plainTextEdit_2->toPlainText().toUtf8();
  char *err = nullptr;

  if (text.isEmpty()) {
    return;
  }

  if (!m_data) {
    delete m_data;
    m_data = nullptr;
  }

#if defined(USE_DND_RAW_DATA)
  auto buf = RiamifyAppGenerateWaveform(m_app, text.data(), &err);

  if (err != nullptr) {
      QString message = QString::fromUtf8(err);
      invokeErrorMessage(message);
      RiamifyReleasePointer(reinterpret_cast<void *>(err));
  }

  if (m_data != nullptr) {
    delete m_data;
    m_data = nullptr;
  }

  m_data = new QMimeData();
  m_data->setData("audio/wav", QByteArray::fromRawData(buf.waveform, static_cast<int>(buf.len)));

  RiamifyAppReleaseWaveform(buf);
#else
  auto buf = RiamifyAppGenerateWaveformIntoFile(m_app, text.data(), &err);
  if (err != nullptr) {
    QString message = QString::fromUtf8(err);
    invokeErrorMessage(message);
    RiamifyReleasePointer(reinterpret_cast<void *>(err));
  }

  // auto clipboard = QApplication::clipboard();
  m_data = new QMimeData();
  m_data->setData("text/uri-list", buf);
#endif
}

static void removeText(QPlainTextEdit *textEdit) {
  auto cursor = textEdit->textCursor();

  cursor.beginEditBlock();

  cursor.select(QTextCursor::Document);
  cursor.removeSelectedText();

  cursor.endEditBlock();

  textEdit->setTextCursor(cursor);
}

void MainWindow::resetField() {
  // Shift+クリックの際は完全に空にする
  auto modifiers = QGuiApplication::keyboardModifiers();

  if (modifiers.testFlag(Qt::ShiftModifier)) {
      ui->plainTextEdit->clear();
      ui->plainTextEdit_2->clear();
  } else {
      removeText(ui->plainTextEdit);
      removeText(ui->plainTextEdit_2);
  }

  ui->horizontalSlider->setValue(100);
}

void MainWindow::speakText() {
  auto text = ui->plainTextEdit_2->toPlainText().toUtf8();
  if (text.isEmpty()) {
    return;
  }

  char *err = nullptr;
  RiamifyAppSpeak(m_app, text.data(), &err);

  if (err != nullptr) {
    QString message = QString::fromUtf8(err);
    invokeErrorMessage(message);
    RiamifyReleasePointer(reinterpret_cast<void *>(err));
  }
}

void MainWindow::invokeErrorMessage(QString const& message) {
  ui->statusBar->setStyleSheet("color: #ffaaaa;\nfont: 9pt \"Yu Gothic UI\";");
  ui->statusBar->showMessage(message, 2000);
  connect(ui->statusBar, SIGNAL(messageChanged(QString)), this, SLOT(resetDisplay()));
}

void MainWindow::resetDisplay() {
  disconnect(ui->statusBar, SIGNAL(messageChanged(QString)), this, SLOT(resetDisplay()));
  ui->statusBar->setStyleSheet("color: white;\nfont: 9pt \"Yu Gothic UI\";");
}

void MainWindow::setClipboardPasteEnabled(bool) {
  //
}

void MainWindow::setClipboardCopyEnabled(bool) {
  //
}

#if defined(Q_OS_WIN)
static void openWithNotepad(HWND hwnd, const wchar_t *filename) {
    SHELLEXECUTEINFO sei = { 0 };
    WCHAR buf[MAX_PATH];
    GetCurrentDirectory(MAX_PATH + 1, reinterpret_cast<LPWSTR>(buf));

    sei.hwnd = hwnd;
    sei.cbSize = sizeof(SHELLEXECUTEINFO);
    sei.nShow = SW_SHOWNORMAL;
    sei.fMask = SEE_MASK_NO_CONSOLE;
    sei.lpFile = L"notepad.exe";
    sei.lpParameters = filename;
    sei.lpDirectory = buf;

    ShellExecuteEx(&sei);
    WaitForSingleObject(sei.hProcess, 10);

    CloseHandle(sei.hProcess);

}
#endif

void MainWindow::openSettings() {
#if defined(Q_OS_WIN)
  openWithNotepad(HWND(nullptr),
               L"settings.yml");

#elif defined(Q_OS_MAC)
  system("open -t settings.yml");
#endif
}

void MainWindow::managePresets() {
#if defined(Q_OS_WIN)
  WCHAR buf[MAX_PATH];
  GetCurrentDirectory(MAX_PATH + 1, reinterpret_cast<LPWSTR>(buf));


  openWithNotepad(HWND(nullptr),
               L"presets.yml");
#elif defined(Q_OS_MAC)
  system("open -t presets.yml");
#endif
}
