use log::Level;
use pretty_env_logger::{formatted_builder};
use pretty_env_logger::env_logger::fmt::{Color, Style, StyledValue};
use pretty_env_logger::env_logger::Target;

const LOG_ENV_VAR: &str = "RUST_LOG";

/// Initialize application logger.
pub fn init_log() {
    let mut builder = formatted_builder();
    builder.target(Target::Stdout);

    builder.format(|f, record| {
        use std::io::Write;

        let level = record.level();
        let args = record.args();
        let time_prefix = f.timestamp_millis();

        let mut style = f.style();
        let level_prefix = colored_level(&mut style, level);

        match record.level() {
            Level::Error | Level::Warn => writeln!(f, "{}: {}", level_prefix, args),
            Level::Info => writeln!(f, "{args}"),
            Level::Debug | Level::Trace => writeln!(f, "[{}] {}: {}", time_prefix, level_prefix, args),
        }
    });

    match std::env::var(LOG_ENV_VAR) {
        Ok(s) => builder.parse_filters(&s),
        Err(_) => builder.parse_filters("info")
    };

    builder.init()
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO"),
        Level::Warn => style.set_color(Color::Yellow).value("WARN"),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}
