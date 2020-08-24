#include <QMessageBox>

#include "../riamify/riamify.h"

#ifdef Q_OS_WIN

#include <Windows.h>
#include <mmsystem.h>

extern "C" void ___PlayWavRiamu(const char *buf, size_t size) {
    PlaySound(reinterpret_cast<LPCWSTR>(buf), nullptr, SND_MEMORY | SND_ASYNC);
}
#endif

// エラーを表示するためにいちいちエラー情報を渡すのはめんどうなので
extern "C" void ___RiamifyShowAlert(Slice message) {
    auto len = message.len;
    auto ptr = message.ptr;

    QMessageBox::critical(nullptr,
                          QMessageBox::tr("Critical Error"),
                          QString::fromUtf8(reinterpret_cast<const char*>(ptr), static_cast<int>(len)));
}
