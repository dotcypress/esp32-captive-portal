use crate::captive::CaptivePortal;
use dns::*;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
    http::{
        server::{Configuration, EspHttpServer},
        Method,
    },
    io::Write,
    ipv4::{self, Mask, RouterConfiguration, Subnet},
    log::EspLogger,
    netif::{EspNetif, NetifConfiguration, NetifStack},
    nvs::EspDefaultNvsPartition,
    sys::{self, EspError},
    wifi::{self, AccessPointConfiguration, EspWifi, WifiDriver},
};
use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

mod captive;
mod dns;

pub const IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(192, 168, 42, 1);

fn main() -> Result<(), EspError> {
    unsafe {
        sys::nvs_flash_init();
    }
    sys::link_patches();
    EspLogger::initialize_default();

    let event_loop = EspSystemEventLoop::take()?;
    let peripherals = Peripherals::take()?;

    log::info!("Starting Wi-Fi...");
    let wifi_driver = WifiDriver::new(
        peripherals.modem,
        event_loop.clone(),
        EspDefaultNvsPartition::take().ok(),
    )?;
    let mut wifi = EspWifi::wrap_all(
        wifi_driver,
        EspNetif::new(NetifStack::Sta)?,
        EspNetif::new_with_conf(&NetifConfiguration {
            ip_configuration: ipv4::Configuration::Router(RouterConfiguration {
                subnet: Subnet {
                    gateway: IP_ADDRESS,
                    mask: Mask(24),
                },
                dhcp_enabled: true,
                dns: Some(IP_ADDRESS),
                secondary_dns: Some(IP_ADDRESS),
            }),
            ..NetifConfiguration::wifi_default_router()
        })?,
    )
    .expect("WiFi init failed");

    wifi.set_configuration(&wifi::Configuration::AccessPoint(
        AccessPointConfiguration {
            ssid: env!("SSID").into(),
            password: env!("SSID_PASSWORD").into(),
            auth_method: wifi::AuthMethod::WPA2Personal,
            ..Default::default()
        },
    ))?;
    wifi.start()?;
    log::info!("Wi-Fi started");

    log::info!("Starting DNS server...");
    let mut dns = SimpleDns::try_new(IP_ADDRESS).expect("DNS server init failed");
    thread::spawn(move || loop {
        dns.poll().ok();
        sleep(Duration::from_millis(50));
    });
    log::info!("DNS server started");

    let store = Arc::new(Mutex::new(String::new()));

    log::info!("Starting HTTP server...");
    let config = Configuration::default();
    let mut server = EspHttpServer::new(&config).expect("HTTP server init failed");
    CaptivePortal::attach(&mut server, IP_ADDRESS).expect("Captive portal attach failed");

    server.fn_handler("/styles.css", Method::Get, |request| {
        request
            .into_response(200, None, &[("Content-Type", "text/css; charset=utf-8")])?
            .write_all(include_bytes!("web/styles.css"))?;
        Ok(())
    })?;

    let memo = store.clone();
    server.fn_handler("/", Method::Get, move |request| {
        let page = format!(include_str!("web/index.html"), memo.lock()?);
        request.into_ok_response()?.write_all(page.as_bytes())?;
        Ok(())
    })?;

    let memo = store.clone();
    server.fn_handler("/", Method::Post, move |mut request| {
        let mut scratch = [0; 256];
        let len = request.read(&mut scratch)?;
        let req = std::str::from_utf8(&scratch[0..len])?;
        if let Some(("memo", req)) = req.split_once('=') {
            *memo.lock()? = urlencoding::decode(req)?.into_owned();
        };
        request.into_response(302, None, &[("Location", "/")])?;
        Ok(())
    })?;

    log::info!("HTTP server started");

    loop {
        sleep(Duration::from_millis(100));
    }
}
