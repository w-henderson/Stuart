//! Provides logging functionality.

mod progress;

pub use progress::Progress;

use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// The global logger.
pub static LOGGER: OnceCell<Logger> = OnceCell::new();

/// The core logger, encapsulating logging configuration and implementing methods for logging.
#[derive(Debug)]
pub struct Logger {
    /// The level of logging to perform.
    pub level: LogLevel,
    /// Whether the logger is enabled.
    pub enabled: AtomicBool,
    /// Whether the logger has logged anything.
    pub has_logged: AtomicBool,
}

/// The log level, which determines the verbosity of logging.
#[derive(Debug, PartialEq, Eq)]
pub enum LogLevel {
    /// Minimal logging.
    Quiet,
    /// Normal logging.
    Normal,
    /// Verbose logging.
    Verbose,
}

impl Logger {
    /// Creates a new logger at the given log level.
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            enabled: AtomicBool::new(true),
            has_logged: AtomicBool::new(false),
        }
    }

    /// Registers the logger as the global logger.
    pub fn register(self) {
        LOGGER.set(self).unwrap();
    }

    /// Returns `true` if the logger has logged anything.
    pub fn has_logged(&self) -> bool {
        self.has_logged.load(Ordering::SeqCst)
    }
}

/// Logs a message.
///
/// The first argument is the verb, which appears in green text.
/// The rest of the arguments are the same as in the `format!` macro.
#[macro_export]
macro_rules! log {
    ($verb:expr, $($arg:tt)*) => {
        if let Some(logger) = $crate::logger::LOGGER.get() {
            if logger.enabled.load(::std::sync::atomic::Ordering::Relaxed)
                && logger.level != $crate::logger::LogLevel::Quiet {
                use ::termcolor::*;
                use std::io::Write;

                let writer = BufferWriter::stderr(ColorChoice::Always);
                let mut buffer = writer.buffer();

                buffer
                    .set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_intense(true))
                    .unwrap();
                write!(buffer, "{:>12} ", $verb).unwrap();
                buffer.reset().unwrap();

                writeln!(buffer, $($arg)*).unwrap();

                writer.print(&buffer).unwrap();

                logger.has_logged.store(true, ::std::sync::atomic::Ordering::SeqCst);
            }
        }
    };
}
