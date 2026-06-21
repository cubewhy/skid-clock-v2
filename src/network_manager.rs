use esp_idf_svc::sntp::EspSntp;
use esp_idf_svc::wifi::{AuthMethod, EspWifi};
use std::sync::{Arc, Mutex};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NetState {
    Idle,
    Scanning,
    SelectNetwork,
    InputSSID,
    InputPassword,
    Connecting,
    Connected,
    NtpSyncing,
    NtpSuccess,
    Error(&'static str),
}

/// A globally shareable network interface controller
#[derive(Clone)]
pub struct NetworkController {
    pub wifi: Arc<Mutex<EspWifi<'static>>>,
    pub sntp: Arc<Mutex<Option<EspSntp<'static>>>>,
    pub state: Arc<Mutex<NetState>>,
    pub scan_results: Arc<Mutex<Vec<(String, AuthMethod)>>>,
}

impl NetworkController {
    pub fn new(
        modem: esp_idf_svc::hal::modem::Modem<'static>,
        sys_loop: esp_idf_svc::eventloop::EspSystemEventLoop,
        nvs: esp_idf_svc::nvs::EspDefaultNvsPartition,
    ) -> anyhow::Result<Self> {
        let wifi = EspWifi::new(modem, sys_loop, Some(nvs)).inspect_err(|&e| {
            log::error!("Failed to initialize hardware EspWifi driver: {:?}", e);
        })?;

        Ok(Self {
            wifi: Arc::new(Mutex::new(wifi)),
            sntp: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(NetState::Idle)),
            scan_results: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Helper to check connection status quickly from any app
    pub fn is_connected(&self) -> bool {
        if let Ok(state) = self.state.lock() {
            matches!(
                *state,
                NetState::Connected | NetState::NtpSyncing | NetState::NtpSuccess
            )
        } else {
            false
        }
    }

    pub fn set_state(&self, new_state: NetState) {
        if let Ok(mut lock) = self.state.lock() {
            *lock = new_state;
        }
    }
}
