use esp_idf_sys::{self as _, esp_restart};// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use anyhow::{anyhow, Result};
use embedded_hal::delay::blocking::DelayUs;
use esp_idf_hal::delay;
use log::{info, error};
use std::{sync::Arc, time::Duration, thread};
use esp_idf_svc::{netif::EspNetifStack, sysloop::EspSysLoopStack, nvs::EspDefaultNvs, wifi::EspWifi};
use embedded_svc::wifi::*;

mod nvs;
mod netcfg;

fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("system start...");

    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new().expect("Unable to init Netif Stack"));
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new().expect("Unable to init sys_loop"));

    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    let mut delay = delay::Ets {};

    delay.delay_us(100_u32)?;

    //删除保存的wifi密码(测试)
    // netcfg::clear_ssid_password()?;

    // 读取保存的wifi密码
    let mut netcfg = None;

    match netcfg::load_ssid_password(){
        Err(err) => {
            error!("{:?}", err);
        }
        Ok(cfg) => {
            info!("read netcfg from nvs:{:?}", cfg);
            netcfg.replace(cfg);
        }
    }

    if netcfg.is_none(){
        match netcfg::receive_ssid_password(default_nvs.clone(), "Hello-ESP32"){
            Err(err) => {
                error!("receive_ssid_password error:{:?}", err);
                thread::sleep(Duration::from_millis(1000));
                unsafe{ esp_restart() };
            }
            Ok(cfg) => {
                netcfg::save_ssid_password(&cfg)?;
                netcfg.replace(cfg);
            }
        }
    }

    let netcfg = netcfg.unwrap();

    info!("ssid={:?}, password={:?}", netcfg.ssid, netcfg.password);

    let wifi = connect_wifi(netif_stack, sys_loop_stack, default_nvs, &netcfg.ssid, &netcfg.password)?;

    info!("wifi connected: {:?}", wifi.get_status());
    

    loop{
        test_https_client()?;
        thread::sleep(Duration::from_millis(10*1000));
    }
}

fn connect_wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
    ssid: &str,
    pass: &str,
) -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.into(),
        password: pass.into(),
        ..Default::default()
    }))?;

    info!("Wifi configuration set, about to get status");

    wifi.wait_status_with_timeout(Duration::from_secs(20), |status| !status.is_transitional())
        .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Stopped,
    ) = status
    {
        info!("Wifi connected: {:?}", ip_settings);
        Ok(wifi)
    } else {
        Err(anyhow!("Unexpected Wifi status: {:?}", status))
    }
}

fn test_https_client() -> Result<()> {
    use embedded_svc::http::client::*;
    use embedded_svc::io;
    use esp_idf_svc::http::client::*;

    let url = String::from("http://www.weather.com.cn/data/sk/101010100.html");

    info!("About to fetch content from {}", url);

    let mut client = EspHttpClient::new(&EspHttpClientConfiguration {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

        ..Default::default()
    })?;

    let mut response = client.get(&url)?.submit()?;

    let mut body = [0_u8; 3048];

    let (body, _) = io::read_max(response.reader(), &mut body)?;

    info!(
        "Body (truncated to 3K):\n{:?}",
        String::from_utf8_lossy(body).into_owned()
    );

    Ok(())
}