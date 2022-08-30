use std::io::Write;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

pub struct Progress {
    name: String,
    total: usize,
    current: usize,
}

impl Progress {
    pub fn new(name: impl AsRef<str>, total: usize) -> Self {
        Self {
            name: name.as_ref().to_string(),
            total,
            current: 0,
        }
    }

    pub fn print(&self) {
        let writer = BufferWriter::stderr(ColorChoice::Always);
        let mut buffer = writer.buffer();

        buffer
            .set_color(
                ColorSpec::new()
                    .set_fg(Some(Color::Green))
                    .set_intense(true),
            )
            .unwrap();
        write!(buffer, "\r{:>12} ", self.name).unwrap();
        buffer.reset().unwrap();

        let scaled_length = ((self.current as f64 / self.total as f64) * 50.0).ceil() as usize;

        write!(
            buffer,
            "[{:<50}] {}/{}",
            "=".repeat(scaled_length),
            self.current,
            self.total
        )
        .unwrap();

        writer.print(&buffer).unwrap();
    }
}

impl Iterator for Progress {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.total {
            return None;
        }

        self.current += 1;
        self.print();

        if self.current == self.total {
            println!();
        }

        Some(())
    }
}
