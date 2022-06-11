use rodio::{
    Sink,
    OutputStream
};

pub struct Audio {
    sink: Sink,
    _stream: OutputStream
}

impl Audio {
    pub fn new() -> Result<Audio, String> {
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok(v) => v,
            Err(err) => { return Err(err.to_string()); }
        };
        let sink = match Sink::try_new(&stream_handle) {
            Ok(v) => v,
            Err(err) => { return Err(err.to_string()); }
        };
        sink.append(rodio::source::SineWave::new(440.0));
        sink.pause();
        let ret = Audio {sink, _stream: stream};
        Ok(ret)
    }

    pub fn play(&self) {
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }
}
