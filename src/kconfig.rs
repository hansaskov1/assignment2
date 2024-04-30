use std::collections::HashMap;

#[derive(Debug)]
pub struct ProjBuild<'a> {
    pub wifi_ssid: &'a str,
    pub wifi_password: &'a str,
    pub mqtt_broker: &'a str,
    pub mqtt_command_topic: &'a str,
    pub mqtt_response_topic: &'a str,
}

impl<'a> ProjBuild<'a> {
    pub fn parse(input: &'a str) -> ProjBuild<'a> {
        let mut lines = input.lines();
        let mut config_map = HashMap::new();

        while let Some(line) = lines.next() {
            if line.trim().starts_with("config") {
                log::info!("{line}");
                let key = line.split_once(' ').unwrap().1.to_lowercase();
                log::info!("{key}");
                lines.next(); // Skip the comment
                let line = lines.next().unwrap();
                log::info!("value line :{line}");
                let value = line.trim().split_once(' ').unwrap().1.trim_matches('\"');
                log::info!("{value}");
                config_map.insert(key, value);
            }
        }

        let config = ProjBuild {
            mqtt_broker: config_map.get("mqtt_broker").unwrap(),
            wifi_ssid: config_map.get("wifi_ssid").unwrap(),
            wifi_password: config_map.get("wifi_password").unwrap(),
            mqtt_command_topic: config_map.get("mqtt_command_topic").unwrap(),
            mqtt_response_topic: config_map.get("mqtt_response_topic").unwrap(),
        };

        return config;
    }
}
