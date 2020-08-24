#pragma once

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

struct __RiamifyApp;
typedef struct __RiamifyApp * RiamifyApp;

typedef struct __Slice {
  void *ptr;
  size_t len;
} Slice;

RiamifyApp *RiamifyAppCreate(char **err);
void        RiamifyAppRelease(RiamifyApp *app);
void        RiamifyAppSpeak(RiamifyApp *app, const char *message, char **err);

size_t      RiamifyAppGetPresetsLength(RiamifyApp *app);
Slice       RiamifyAppGetPresetName(RiamifyApp *app, int id);
bool        RiamifyAppUpdatePresets(RiamifyApp *app);

void        RiamifyAppSetPreset(RiamifyApp *app, int id);
void        RiamifyAppSetSpeed(RiamifyApp *app, int speed);

typedef struct __Waveform_Riamify {
  char  *waveform;
  size_t len;
  size_t capacity;
} Waveform;

Waveform    RiamifyAppGenerateWaveform(RiamifyApp *app, const char *message, char **err);
char*       RiamifyAppGenerateWaveformIntoFile(RiamifyApp *app, const char *message, char **err);
void        RiamifyAppReleaseWaveform(Waveform waveform);

int8_t *RiamifyAppGetYomi(RiamifyApp *app, const int8_t *msg);
void    RiamifyReleasePointer(void * ptr);

} // extern "C"
