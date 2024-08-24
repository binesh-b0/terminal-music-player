use rodio::Sink;
use std::time::{Instant, Duration};
use crossterm::{execute, cursor::MoveTo};
use std::io::{stdout, Write};

pub fn adjust_volume(sink: &Sink, increase: bool) {
    let current_volume = sink.volume();
    let new_volume = if increase {
        (current_volume + 0.1).min(1.0)
    } else {
        (current_volume - 0.1).max(0.0)
    };
    sink.set_volume(new_volume);
}

pub fn display_progress(start_time: Instant, track_duration: Duration) {
    let elapsed = start_time.elapsed();
    let remaining = if track_duration > elapsed {
        track_duration - elapsed
    } else {
        Duration::new(0, 0)
    };

    let mut stdout = stdout();
    // Move the cursor instead of clearing the entire screen
    execute!(stdout, MoveTo(0, 0)).unwrap();

    println!(
        "Progress: [{}/{}]",
        format_time(elapsed.as_secs()),
        format_time(track_duration.as_secs())
    );
}

fn format_time(seconds: u64) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

pub fn show_help_menu() {
    println!("Available Commands:");
    println!("P - Play/Pause");
    println!("S - Stop");
    println!("N - Next Track");
    println!("B - Previous Track");
    println!("+ - Increase Volume");
    println!("- - Decrease Volume");
    println!("Q - Quit");
}
