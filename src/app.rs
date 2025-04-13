use shellwords;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::process::{Command, Stdio};
use strsim::jaro_winkler;
use tokio::{time::Duration, time::sleep};

use crate::config;
use crate::event::{AppEvent, Event, EventHandler};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Event handler.
    pub events: EventHandler,
    /// User Input
    pub user_input: String,
    /// Results
    pub results: Vec<String>,
    /// Command Bindings
    pub bindings: HashMap<String, String>,
    /// status message
    pub status_message: String,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let bindings = config::Config::from_file("/home/rotted/.config/run/config.toml")
            .map(|c| c.mappings)
            .unwrap_or_default();

        Self {
            running: true,
            user_input: String::new(),
            events: EventHandler::new(),
            results: Vec::new(),
            status_message: String::new(),
            bindings,
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => self.handle_key_events(key_event)?,
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit(),
                    AppEvent::Launch => self.launch_closest(),
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Char(c) => {
                self.user_input.push(c);
                self.update_results();
            }
            KeyCode::Backspace => {
                self.user_input.pop();
                self.update_results();
            }
            KeyCode::Enter => {
                self.events.send(AppEvent::Launch);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {}

    pub fn update_results(&mut self) {
        let suggestions = self.closest_bindings(&self.user_input);
        self.results = suggestions.into_iter().map(|(alias, _)| alias).collect();
    }

    pub fn launch_closest(&mut self) {
        if let Some((alias, _)) = self.closest_bindings(&self.user_input).into_iter().next() {
            if let Some(command) = self.bindings.get(&alias) {
                match Self::run_detached(command) {
                    Ok(_) => {
                        self.status_message = format!("Launched '{}'", alias);
                    }
                    Err(e) => {
                        self.status_message = format!("Failed to launch '{}': {}", alias, e);
                    }
                }
            } else {
                self.status_message = format!("No command bound to '{}'", alias);
            }
        } else {
            self.status_message = "No matching command found".to_string();
        }

        self.user_input.clear();
    }

    pub fn close_terminal_emulator() {
        // Get our current process ID
        let pid = std::process::id();

        // Read /proc/[pid]/status to get PPid
        let status_path = format!("/proc/{}/status", pid);
        if let Ok(contents) = fs::read_to_string(status_path) {
            for line in contents.lines() {
                if line.starts_with("PPid:") {
                    if let Some(ppid_str) = line.split_whitespace().nth(1) {
                        if let Ok(ppid) = ppid_str.parse::<u32>() {
                            // Kill the parent process
                            let _ = Command::new("kill")
                                .arg("-9")
                                .arg(ppid.to_string())
                                .stdin(Stdio::null())
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .spawn();
                        }
                    }
                    break;
                }
            }
        }
    }

    /// Runs a terminal command detached from the terminal application
    pub fn run_detached(command: &str) -> io::Result<()> {
        // Split the command line into command and args
        let mut parts = shellwords::split(command)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        if parts.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Empty command"));
        }

        let program = parts.remove(0);

        Command::new(program)
            .args(parts)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        Ok(())
    }
    pub fn closest_bindings(&self, input: &str) -> Vec<(String, f64)> {
        let mut scored: Vec<_> = self
            .bindings
            .keys()
            .map(|alias| (alias.clone(), jaro_winkler(input, alias)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(5).collect()
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
        Self::close_terminal_emulator();
    }
}
