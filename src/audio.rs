use rodio::{
    Sink
};

pub struct Audio {
    sink: Sink
}

impl Audio {
    pub fn new() -> Option<Audio> {
        let dev = rodio::default_output_device()?;
        let sink = Sink::new(&dev);
        sink.append(rodio::source::SineWave::new(440));
        sink.pause();
        let ret = Audio {sink};
        Some(ret)
    }

    pub fn play(&self) {
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }
}
