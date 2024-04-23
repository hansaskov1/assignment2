use std::{
    convert::TryInto,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct Command {
    pub num_measurements: u16,
    pub interval_ms: u16,
}

impl TryInto<Command> for &[u8] {
    type Error = &'static str;

    fn try_into(self) -> Result<Command, Self::Error> {
        let s = std::str::from_utf8(self).map_err(|_| "Invalid UTF-8")?;

        let (delimiter, data) = s
            .split_once(':')
            .ok_or("Invalid format, expected ':' separator")?;

        if delimiter != "measure" {
            return Err("Invalid format, expected \"measure\" as the first character");
        }

        let (num_measurements, interval_ms) = data
            .split_once(',')
            .ok_or("Invalid format, expected ',' separator")?;

        Ok(Command {
            num_measurements: num_measurements
                .parse()
                .map_err(|_| "Invalid num_measurements")?,
            interval_ms: interval_ms.parse().map_err(|_| "Invalid interval_ms")?,
        })
    }
}
