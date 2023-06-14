use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};
use log::*;

use apps::wifi::*;
use apps::*;

pub fn start() {
    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let notifier = sch::SCH1HZ.subscribe();

    let mut out = Out::default();

    info!("started wifi");

    let mut wifi_driver = Box::new(EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap());
    wifi_driver
        .set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: SSID.into(),
            bssid: None,
            auth_method: embedded_svc::wifi::AuthMethod::WPA2Personal,
            password: PASSWORD.into(),
            channel: None,
        }))
        .unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();

    loop {
        notifier.wait();

        let has_ip = !wifi_driver
            .sta_netif()
            .get_ip_info()
            .unwrap()
            .ip
            .is_unspecified();

        if !out.has_ip && has_ip {
            info!("connected");
            tcp::CMD_QUEUE.push(tcp::Command::Enable);
        } else if out.has_ip && !has_ip {
            info!("disconnected");
            tcp::CMD_QUEUE.push(tcp::Command::Disable);
        }
        out.has_ip = has_ip;
        OUT_DATA.send(out);
        to::SEND_QUEUE.push(SbMsg::WifiOut(out));
    }
}
