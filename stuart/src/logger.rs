use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};

pub static LOGGER: OnceCell<Logger> = OnceCell::new();

#[derive(Debug)]
pub struct Logger {
    pub level: LogLevel,
    pub has_logged: AtomicBool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LogLevel {
    Quiet,
    Normal,
    Verbose,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            has_logged: AtomicBool::new(false),
        }
    }

    pub fn register(self) {
        LOGGER.set(self).unwrap();
    }

    pub fn global() -> &'static Logger {
        LOGGER.get().unwrap()
    }

    pub fn has_logged(&self) -> bool {
        self.has_logged.load(Ordering::SeqCst)
    }
}

#[macro_export]
macro_rules! log {
    ($verb:expr, $($arg:tt)*) => {
        if let Some(logger) = $crate::logger::LOGGER.get() {
            if logger.level != $crate::logger::LogLevel::Quiet {
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
