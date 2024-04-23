use std::sync::mpsc;
use std::time::Instant;
use std::{thread, time::Duration};

use assignment2::adc::AdcTempReader;
use assignment2::command::Command;
use esp_idf_hal::adc::*;
use esp_idf_hal::{adc::config::Config, peripherals::Peripherals};
use esp_idf_svc::mqtt::client::{EventPayload, QoS};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    mqtt::client::{EspMqttClient, EspMqttConnection, MqttClientConfiguration},
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

// NOTICE: Change this to your WiFi network SSID
const WIFI_SSID: &str = "hansaskov";
const WIFI_PASSWORD: &str = "hansaskov";

// NOTICE: Change this to your MQTT broker URL, make sure the broker is on the same network as you
const MQTT_BROKER: &str = "mqtt://192.168.232.62:1883";
const MQTT_COMMAND_TOPIC: &str = "command";
const MQTT_RESPONSE_TOPIC: &str = "response";

fn main() {
    let start_time = Instant::now();

    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    // Configure Wifi
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
        sys_loop,
    )
    .unwrap();

    // Establish connection to WiFi network
    connect_wifi(&mut wifi).unwrap();

    let adc = AdcDriver::new(peripherals.adc1, &Config::new().calibration(true)).unwrap();
    let adc_pin: AdcChannelDriver<{ attenuation::DB_6 }, _> =
        AdcChannelDriver::new(peripherals.pins.gpio34).unwrap();
    let mut adc_temp_reader = AdcTempReader::new(adc, adc_pin).unwrap();

    // Configure MQTT client
    let (mut mqtt_client, mut mqtt_conn) = mqtt_create(MQTT_BROKER, MQTT_COMMAND_TOPIC).unwrap();

    std::thread::scope(|s| {
        let (tx, rx) = mpsc::channel::<Command>();

        std::thread::Builder::new()
            .stack_size(6000)
            .spawn_scoped(s, move || {
                log::info!("MQTT Listening for messages");
                mqtt_client
                    .subscribe(MQTT_RESPONSE_TOPIC, QoS::AtLeastOnce)
                    .unwrap();

                while let Ok(event) = mqtt_conn.next() {
                    if let EventPayload::Received { topic, data, .. } = event.payload() {
                        if topic == Some(MQTT_COMMAND_TOPIC) {
                            if let Ok(command) = data.try_into() {
                                tx.send(command).unwrap();
                            }
                        }
                    }
                }

                loop {

                    if let Ok(command) = rx.recv() {
                        let duration_interval = Duration::from_millis(command.interval_ms.into());
                        for i in command.num_measurements..0 {
                            let start_response = Instant::now();
                            let temperature = adc_temp_reader.read_temperature().unwrap();
                            let uptime = get_uptime(start_time);

                            // Publish MQTT message
                            let msg = format!("{i},{temperature},{uptime}");
                            mqtt_client
                                .publish(
                                    MQTT_RESPONSE_TOPIC,
                                    QoS::AtLeastOnce,
                                    false,
                                    msg.as_bytes(),
                                )
                                .unwrap();

                            // Duration interval - time spend sending the response
                            let sleep_duration = duration_interval - start_response.elapsed();

                            if sleep_duration > Duration::from_millis(0) {
                                thread::sleep(sleep_duration);
                            }
                        }
                    }
                }
            })
            .unwrap();
    })
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.try_into().unwrap(),
        auth_method: AuthMethod::WPA2WPA3Personal,
        password: WIFI_PASSWORD.try_into().unwrap(),
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

fn mqtt_create(
    url: &str,
    topic: &str,
) -> anyhow::Result<(EspMqttClient<'static>, EspMqttConnection)> {
    log::info!("Starting MQTT client");
    let (mut mqtt_client, mqtt_conn) = EspMqttClient::new(
        url,
        &MqttClientConfiguration {
            ..Default::default()
        },
    )?;

    mqtt_client.subscribe(topic, esp_idf_svc::mqtt::client::QoS::AtLeastOnce)?;

    log::info!("MQTT client connected");

    Ok((mqtt_client, mqtt_conn))
}

fn get_uptime(start_time: Instant) -> u128 {
    start_time.elapsed().as_millis()
}
