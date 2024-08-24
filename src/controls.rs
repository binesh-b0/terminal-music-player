use rodio::Sink;

/// Adjusts the volume of the audio playback.
///
/// # Arguments
///
/// * `sink` - A reference to the `Sink` which controls audio playback.
/// * `increase` - A boolean value that determines whether to increase (`true`) or decrease (`false`) the volume.
pub fn adjust_volume(sink: &Sink, increase: bool) {
    // Get the current volume level of the sink.
    let current_volume = sink.volume();
    
    // Calculate the new volume based on whether the volume should be increased or decreased.
    let new_volume = if increase {
        // Increase the volume by 0.1, but ensure it doesn't exceed the maximum value of 1.0.
        (current_volume + 0.1).min(1.0)
    } else {
        // Decrease the volume by 0.1, but ensure it doesn't drop below the minimum value of 0.0.
        (current_volume - 0.1).max(0.0)
    };
    
    // Set the sink's volume to the newly calculated value.
    sink.set_volume(new_volume);
}
