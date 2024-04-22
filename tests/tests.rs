#[cfg(test)]
#[cfg(not(target_arch = "xtensa"))]
mod tests {
    use assignment2::command::Command;

    use super::*;

    #[test]
    fn test_valid_command() {
        let bytes = b"command:10,100";
        let command: Command = (&bytes[..]).try_into().unwrap();
        assert_eq!(command.num_measurements, 10);
        assert_eq!(command.interval_ms, 100);
    }

    #[test]
    fn test_invalid_format() {
        let bytes = b"invalid command";
        let result: Result<Command, _> = (&bytes[..]).try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_numbers() {
        let bytes = b"command:abc,def";
        let result: Result<Command, _> = (&bytes[..]).try_into();
        assert!(result.is_err());
    }
}
