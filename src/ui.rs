use crossterm::{execute, terminal::{Clear, ClearType}, cursor::MoveTo, style::Stylize};
use std::io::{stdout, Write};
use std::time::Duration;

/// Clears the entire terminal screen.
///
/// This function uses the `crossterm` crate to clear the terminal, providing a clean slate
/// for subsequent UI updates.
pub fn clear_screen() {
    let mut stdout = stdout();
    // Clear the entire terminal screen.
    execute!(stdout, Clear(ClearType::All)).unwrap();
}

/// Moves the terminal cursor to a specified position.
///
/// # Arguments
///
/// * `x` - The horizontal position (column) to move the cursor to.
/// * `y` - The vertical position (row) to move the cursor to.
pub fn move_cursor_to(x: u16, y: u16) {
    let mut stdout = stdout();
    // Move the cursor to the specified (x, y) position.
    execute!(stdout, MoveTo(x, y)).unwrap();
}

/// Displays the metadata of the current track on the terminal.
///
/// # Arguments
///
/// * `title` - The title of the track.
/// * `duration` - The duration of the track as a `Duration`.
/// * `file_name` - The name of the audio file being played.
pub fn display_metadata(title: &str, duration: Duration, file_name: &str) {
    // Move the cursor to the position where metadata should be displayed.
    move_cursor_to(2, 1);
    // Print the file name, title, and duration, with styling.
    println!(
        "{}\n{} - Duration: {}",
        file_name.blue().bold(),                    // Style the file name in blue and bold.
        title.green().bold(),                       // Style the title in green and bold.
        format_time(duration.as_secs()).blue()      // Format and style the duration in blue.
    );
}

/// Displays a progress bar indicating the playback progress of the current track.
///
/// # Arguments
///
/// * `elapsed` - The amount of time that has elapsed since the track started playing.
/// * `total` - The total duration of the track.
pub fn display_progress_bar(elapsed: Duration, total: Duration) {
    let progress_length = 50; // The total length of the progress bar.
    // Calculate the progress as a fraction of the total duration.
    let progress = elapsed.as_secs_f32() / total.as_secs_f32();
    // Determine how many characters of the progress bar should be filled.
    let filled_length = (progress * progress_length as f32) as usize;
    // Create the progress bar string.
    let bar = "█".repeat(filled_length) + &" ".repeat(progress_length - filled_length);

    // Move the cursor to the position where the progress bar should be displayed.
    move_cursor_to(0, 3);
    // Print the progress bar along with elapsed and total time, with styling.
    println!(
        "[{}] {} / {}",
        bar.yellow(),                                // Style the filled portion of the bar in yellow.
        format_time(elapsed.as_secs()).white(),      // Format and style the elapsed time in white.
        format_time(total.as_secs()).white()         // Format and style the total time in white.
    );
}

/// Displays an animated spinner to indicate ongoing playback.
///
/// # Arguments
///
/// * `spinner_pos` - The current position in the spinner animation cycle.
pub fn display_spinner(spinner_pos: usize) {
    // Define the frames of the spinner animation.
    const SPINNERS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    // Move the cursor to the position where the spinner should be displayed.
    move_cursor_to(0, 1);
    // Print the current frame of the spinner animation, with cyan styling.
    print!("{}", SPINNERS[spinner_pos % SPINNERS.len()].cyan());
    // Flush the stdout to ensure the spinner is displayed immediately.
    stdout().flush().unwrap();
}

/// Displays the key bindings available to the user on the terminal.
///
/// This function positions the key bindings below the other UI components
/// and provides a reference for controlling playback.
pub fn display_key_bindings() {
    // Move the cursor to a fixed position for displaying key bindings.
    move_cursor_to(0, 6);
    // Print the key bindings with magenta and bold styling.
    println!(
        "{}",
        "[q] Quit | [p] Play/Pause | [+] Increase Volume | [-] Decrease Volume"
            .magenta()
            .bold()
    );
}

/// Formats a given time in seconds into a `MM:SS` format string.
///
/// # Arguments
///
/// * `seconds` - The time in seconds to format.
///
/// # Returns
///
/// A string representing the formatted time.
fn format_time(seconds: u64) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)  // Format time as MM:SS.
}
