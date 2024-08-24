use rodio::Sink;

pub fn adjust_volume(sink: &Sink, increase: bool) {
    let current_volume = sink.volume();
    let new_volume = if increase {
        (current_volume + 0.1).min(1.0)
    } else {
        (current_volume - 0.1).max(0.0)
    };
    sink.set_volume(new_volume);
}
