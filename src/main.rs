use std::process::Command;
use std::io::{stdout, Stdout, Write};
use crossterm::{
    cursor::{Hide, MoveTo, Show, position},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

fn main() {
    // Run xrandr --query to get display info
    let output = Command::new("xrandr")
        .arg("--query")
        .output()
        .expect("Failed to run xrandr. Ensure x11 is installed");

    // Convert the raw bytes to a string
    let xrandr_str = String::from_utf8_lossy(&output.stdout).to_string();

    // Parse connected outputs and primary
    let mut connected_outputs: Vec<String> = Vec::new();
    let mut primary = String::new();

    // Split into lines and parse
    for line in xrandr_str.lines() {
        if line.contains(" connected") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let name = parts[0].to_string();

            connected_outputs.push(name.clone());

            if line.contains("primary") {
                primary = name;
            }
        }
    }

    if primary.is_empty() {
        println!("No primary display found. Exiting.");
        return;
    }

    // Filter out the primary from connecteds (we'll attach others to it)
    connected_outputs.retain(|x| x != &primary);

    if connected_outputs.is_empty() {
        println!("No other connected displays found. Exiting.");
        return;
    }

    // Interactive loop
    println!("Primary display: {}", primary);
    loop {
        if connected_outputs.is_empty() {
            println!("No more displays left.");
            break;
        }

        // Select display
        let display_options: Vec<&str> = connected_outputs.iter().map(|s| s.as_str()).collect();
        let selected_display = select_option("Select display to attach (arrow keys to move, enter to select, q to quit):", &display_options);

        let Some(selected) = selected_display else {
            println!("Quitting.");
            break;
        };

        // Select position
        let position_options = vec!["left", "right", "above", "below"];
        let selected_pos = select_option("Select position (arrow keys to move, enter to select, q to quit):", &position_options);

        let Some(pos) = selected_pos else {
            println!("Quitting.");
            break;
        };

        let pos_arg = match pos.as_str() {
            "left" => "left-of",
            "right" => "right-of",
            "above" => "above",
            "below" => "below",
            _ => unreachable!(),
        };

        // Build and run the xrandr command
        let args = vec![
            "--output".to_string(),
            selected.clone(),
            "--auto".to_string(),
            format!("--{}", pos_arg),
            primary.clone(),
        ];

        println!("Running: xrandr {:?}", args);

        let status = Command::new("xrandr")
            .args(&args)
            .status()
            .expect("Failed to execute xrandr command");

        if status.success() {
            println!("Display attached successfully.");
            // Remove from list to avoid re-attaching
            connected_outputs.retain(|x| x != &selected);
        } else {
            println!("Failed to attach display. Check xrandr output for errors.");
        }
    }
}

/// Function to handle arrow key selection
fn select_option(title: &str, options: &[&str]) -> Option<String> {
    if options.is_empty() {
        return None;
    }

    enable_raw_mode().expect("Failed to enable raw mode");
    let mut stdout = stdout();
    execute!(stdout, Hide).expect("Failed to hide cursor");

    // Save the starting cursor position before drawing
    let start_pos = position().expect("Failed to get cursor position");

    let num_lines = 1 + options.len() as u16;

    let mut selected_index: usize = 0;

    // Initial clear and draw
    clear_area(&mut stdout, start_pos, num_lines);
    draw_menu(&mut stdout, start_pos, title, options, selected_index);

    loop {
        // Read key event
        if let Event::Key(KeyEvent { code, modifiers, .. }) = read().expect("Failed to read event") {
            match code {
                KeyCode::Up => {
                    if selected_index > 0 {
                        selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected_index < options.len() - 1 {
                        selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    break;
                }
                KeyCode::Char('q') if modifiers.is_empty() => {
                    clear_area(&mut stdout, start_pos, num_lines);
                    cleanup(&mut stdout);
                    return None;
                }
                KeyCode::Esc => {
                    clear_area(&mut stdout, start_pos, num_lines);
                    cleanup(&mut stdout);
                    return None;
                }
                _ => {}
            }
        }

        // Redraw: Clear area, then draw
        clear_area(&mut stdout, start_pos, num_lines);
        draw_menu(&mut stdout, start_pos, title, options, selected_index);
    }

    // After enter: Clear the menu to remove it after choice
    clear_area(&mut stdout, start_pos, num_lines);

    cleanup(&mut stdout);

    Some(options[selected_index].to_string())
}

fn draw_menu(stdout: &mut Stdout, start_pos: (u16, u16), title: &str, options: &[&str], selected_index: usize) {
    let mut current_row = start_pos.1;

    // Print title
    execute!(stdout, MoveTo(0, current_row), Print(title)).expect("Failed to print title");
    current_row += 1;

    // Print options
    for (i, opt) in options.iter().enumerate() {
        let prefix = if i == selected_index { "> " } else { "  " };
        execute!(stdout, MoveTo(0, current_row), Print(prefix), Print(opt)).expect("Failed to print option");
        current_row += 1;
    }

    stdout.flush().expect("Failed to flush");
}

fn clear_area(stdout: &mut Stdout, start_pos: (u16, u16), num_lines: u16) {
    for i in 0..num_lines {
        execute!(stdout, MoveTo(0, start_pos.1 + i), Clear(ClearType::CurrentLine)).expect("Failed to clear line");
    }
    execute!(stdout, MoveTo(0, start_pos.1)).expect("Failed to reset cursor");
    stdout.flush().expect("Failed to flush");
}

fn cleanup(stdout: &mut Stdout) {
    execute!(stdout, Show).expect("Failed to show cursor");
    disable_raw_mode().expect("Failed to disable raw mode");
}