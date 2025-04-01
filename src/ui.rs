use console::style;
use dialoguer::{Confirm, Input, Select};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::error;
use prettytable::{Cell, Row, Table};

use crate::error::BasecampResult;

/// Terminal UI utilities
pub struct UI;

impl UI {
    /// Print a success message
    pub fn success(message: &str) {
        println!("{} {}", style("✓").green().bold(), message);
    }

    /// Print an error message
    pub fn error(message: &str) {
        eprintln!("{} {}", style("✗").red().bold(), style(message).red());
    }

    /// Print a warning message
    pub fn warning(message: &str) {
        println!("{} {}", style("!").yellow().bold(), message);
    }

    /// Print an info message
    pub fn info(message: &str) {
        println!("{} {}", style("i").blue().bold(), message);
    }

    /// Ask for user confirmation
    pub fn confirm(message: &str, default: bool) -> BasecampResult<bool> {
        match Confirm::new()
            .with_prompt(message)
            .default(default)
            .show_default(true)
            .interact()
        {
            Ok(confirmed) => Ok(confirmed),
            Err(err) => {
                error!("Failed to get user confirmation: {}", err);
                Ok(default) // Fallback to default on error
            }
        }
    }

    /// Ask for user input with an optional default value
    pub fn input<T>(message: &str, default: Option<T>) -> BasecampResult<T>
    where
        T: std::str::FromStr + std::fmt::Display + Clone,
        <T as std::str::FromStr>::Err: std::fmt::Debug + std::fmt::Display,
    {
        let input = Input::new().with_prompt(message);

        let input = if let Some(default_value) = default {
            input.default(default_value).show_default(true)
        } else {
            input
        };

        match input.interact() {
            Ok(value) => Ok(value),
            Err(err) => {
                error!("Failed to get user input: {}", err);
                Err(crate::error::BasecampError::Generic(format!(
                    "Failed to get user input: {}",
                    err
                )))
            }
        }
    }

    /// Display a selection menu with arrow key navigation
    pub fn select(message: &str, options: &[&str], default_index: Option<usize>) -> BasecampResult<usize> {
        let mut select = Select::new()
            .with_prompt(message)
            .items(options);
            
        if let Some(default) = default_index {
            select = select.default(default);
        }
        
        match select.interact() {
            Ok(selection) => Ok(selection),
            Err(err) => {
                error!("Failed to get user selection: {}", err);
                Err(crate::error::BasecampError::Generic(format!(
                    "Failed to get user selection: {}",
                    err
                )))
            }
        }
    }

    /// Create a progress bar
    #[allow(dead_code)]
    pub fn progress_bar(len: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_message(message.to_string());
        pb
    }

    /// Create a spinner
    #[allow(dead_code)]
    pub fn spinner(message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    }

    /// Create a multi-progress bar for parallel operations
    #[allow(dead_code)]
    pub fn multi_progress() -> MultiProgress {
        MultiProgress::new()
    }

    /// Create a table for displaying data
    pub fn create_table(headers: Vec<&str>) -> Table {
        let mut table = Table::new();

        // Convert headers to styled cells
        let header_cells: Vec<Cell> = headers
            .iter()
            .map(|h| Cell::new(h).style_spec("bFg"))
            .collect();

        table.set_titles(Row::new(header_cells));
        
        // Use the box format which allows for better text layout
        table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
        
        table
    }

    /// Add a row to a table
    pub fn add_table_row(table: &mut Table, cells: Vec<String>) {
        // Convert strings to cells
        let row_cells: Vec<Cell> = cells
            .iter()
            .map(|c| Cell::new(c))
            .collect();
            
        table.add_row(Row::new(row_cells));
    }

    /// Display a table
    pub fn print_table(table: &Table) {
        table.printstd();
    }
}
