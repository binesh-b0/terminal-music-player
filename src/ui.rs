use crossterm::{execute, terminal::{Clear, ClearType}, cursor::MoveTo, style::Stylize};
use std::io::{stdout, Write};
use std::time::Duration;

pub fn clear_screen() {
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All)).unwrap();
}

pub fn move_cursor_to(x: u16, y: u16) {
    let mut stdout = stdout();
    execute!(stdout, MoveTo(x, y)).unwrap();
}

pub fn display_metadata(title: &str, duration: Duration,file_name:&str) {
    move_cursor_to(2, 1);
    println!(
        "{}\n{} - Duration: {}",
        file_name.blue().bold(),
        title.green().bold(),
        format_time(duration.as_secs()).blue()
    );
}

pub fn display_progress_bar(elapsed: Duration, total: Duration) {
    let progress_length = 50;
    let progress = elapsed.as_secs_f32() / total.as_secs_f32();
    let filled_length = (progress * progress_length as f32) as usize;
    let bar = "█".repeat(filled_length) + &" ".repeat(progress_length - filled_length);

    move_cursor_to(0, 3);
    println!(
        "[{}] {} / {}",
        bar.yellow(),
        format_time(elapsed.as_secs()).white(),
        format_time(total.as_secs()).white()
    );
}

pub fn display_spinner(spinner_pos: usize) {
    const SPINNERS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    move_cursor_to(0, 1);
    print!("{}", SPINNERS[spinner_pos % SPINNERS.len()].cyan());
    stdout().flush().unwrap();
}

pub fn display_key_bindings() {
    move_cursor_to(0, 6); // Fixed position for key bindings
    println!(
        "{}",
        "[q] Quit | [p] Play/Pause | [+] Increase Volume | [-] Decrease Volume"
            .magenta()
            .bold()
    );
}

fn format_time(seconds: u64) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}
