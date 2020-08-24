pub fn play_wav(waveform: Vec<u8>) {
    extern "C" {
        fn ___PlayWavRiamu(waveform: *const u8, size: usize);
    }

    unsafe {
        ___PlayWavRiamu(waveform.as_ptr(), waveform.len());
    }
}
