use std::sync::mpsc;
use std::time::Instant;
use std::{thread, time::Duration};

use assignment2::adc::AdcTempReader;
use assignment2::command::Command;
use assignment2::kconfig::ProjBuild;
use esp_idf_hal::adc::*;
use esp_idf_hal::{adc::config::Config, peripherals::Peripherals};
use esp_idf_svc::mqtt::client::{EventPayload, QoS};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    mqtt::client::{EspMqttClient, EspMqttConnection, MqttClientConfiguration},
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

const KCONFIG: &str = include_str!("../Kconfig.projbuild");

fn main() {
    let start_time = Instant::now();

    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let config: ProjBuild = ProjBuild::parse(KCONFIG);

    log::info!("Using config: {:?}", config);

    // Configure Wifi
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
        sys_loop,
    )
    .unwrap();

    // Establish connection to WiFi network
    connect_wifi(&mut wifi, &config).unwrap();

    // Initialize Analog to digital converter:
    let adc = AdcDriver::new(peripherals.adc1, &Config::new().calibration(true)).unwrap();
    let adc_pin: AdcChannelDriver<{ attenuation::DB_6 }, _> =
        AdcChannelDriver::new(peripherals.pins.gpio34).unwrap();

    let mut adc_temp_reader = AdcTempReader::new(adc, adc_pin).unwrap();

    // Configure MQTT client
    let (mut mqtt_client, mut mqtt_conn) = mqtt_create(config.mqtt_broker).unwrap();

    // Main thread where code is executed in
    std::thread::scope(|s| {
        // Message bus to communicate between threads
        let (command_sender, command_reciever) = mpsc::channel::<Command>();

        // This new thread will listen for incoming messages in mqtt, parse the command and queue it for processing
        std::thread::Builder::new()
            .stack_size(6000)
            .spawn_scoped(s, move || {
                while let Ok(event) = mqtt_conn.next() {
                    if let EventPayload::Received { topic, data, .. } = event.payload() {
                        if topic == Some(config.mqtt_command_topic) {
                            log::info!("Received message {data:?} on {topic:?}");
                            if let Ok(command) = data.try_into() {
                                command_sender.send(command).unwrap();
                            }
                        }
                    }
                }
            })
            .unwrap();

        // Subscribe to command and response topic
        mqtt_client
            .subscribe(config.mqtt_command_topic, QoS::AtMostOnce)
            .unwrap();
        mqtt_client
            .subscribe(config.mqtt_response_topic, QoS::AtMostOnce)
            .unwrap();

        // Add small delay to make sure mqtt starts up correctly
        log::info!("Initializing, wait 0,5 seconds");
        thread::sleep(Duration::from_millis(500));

        // Main loop will listen for new messages in the command_reciever queue and execute them when they arrive.
        loop {
            if let Ok(command) = command_reciever.recv() {
                log::info!("Recieved {command:?}");
                let duration_interval = Duration::from_millis(command.interval_ms.into());

                repeat_with_delay(command.num_measurements, duration_interval, |i| {
                    let temperature = adc_temp_reader.read_temperature().unwrap();
                    let uptime = get_uptime(start_time);
                    let msg = format!("{i},{temperature},{uptime}");

                    log::info!("Sending values: {msg}");

                    mqtt_client
                        .publish(
                            config.mqtt_response_topic,
                            QoS::AtLeastOnce,
                            false,
                            msg.as_bytes(),
                        )
                        .unwrap();
                });
            }
        }
    })
}

fn connect_wifi(
    wifi: &mut BlockingWifi<EspWifi<'static>>,
    config: &ProjBuild,
) -> anyhow::Result<()> {
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: config.wifi_ssid.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        password: config.wifi_password.try_into().unwrap(),
        ..Default::default()
    }))?;

    wifi.start()?;
    log::info!("Wifi started");

    log::info!("Connecting WiFi...");
    wifi.connect()?;
    log::info!("Wifi connected");

    wifi.wait_netif_up()?;
    log::info!("Wifi netif up");

    Ok(())
}

fn mqtt_create(url: &str) -> anyhow::Result<(EspMqttClient<'static>, EspMqttConnection)> {
    log::info!("Starting MQTT client");
    let (mqtt_client, mqtt_conn) = EspMqttClient::new(url, &MqttClientConfiguration::default())?;
    log::info!("MQTT client created");

    Ok((mqtt_client, mqtt_conn))
}

fn get_uptime(start_time: Instant) -> u128 {
    start_time.elapsed().as_millis()
}

fn repeat_with_delay<F, T>(count: u16, delay: Duration, mut send_fn: F)
where
    F: FnMut(u16) -> T,
{
    (0..count).rev().for_each(|i| {
        let start_time = Instant::now();
        send_fn(i);
        let elapsed = start_time.elapsed();

        if let Some(remaining) = delay.checked_sub(elapsed) {
            thread::sleep(remaining);
        }
    });
}
