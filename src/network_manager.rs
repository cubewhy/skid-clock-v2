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

    /// Fetches the current RSSI (Signal Strength in dBm) from the active station interface.
    /// Returns None if disconnected or if the low-level driver query fails.
    pub fn get_rssi(&self) -> Option<i32> {
        if !self.is_connected() {
            return None;
        }

        // Lock to ensure thread-safe hardware telemetry acquisition
        let _wifi_lock = self.wifi.lock().ok()?;

        let mut ap_info = std::mem::MaybeUninit::<esp_idf_svc::sys::wifi_ap_record_t>::uninit();
        unsafe {
            if esp_idf_svc::sys::esp_wifi_sta_get_ap_info(ap_info.as_mut_ptr()) == 0 {
                let ap_info = ap_info.assume_init();
                Some(ap_info.rssi as i32)
            } else {
                None
            }
        }
    }

    pub fn set_state(&self, new_state: NetState) {
        if let Ok(mut lock) = self.state.lock() {
            *lock = new_state;
        }
    }

    /// Dynamically determines and sets the correct fallback state based on hardware status
    pub fn reset_state(&self) -> NetState {
        let fallback = if let Ok(wifi_lock) = self.wifi.lock() {
            if wifi_lock.is_connected().unwrap_or(false) {
                NetState::Connected
            } else {
                NetState::Idle
            }
        } else {
            NetState::Idle
        };

        self.set_state(fallback.clone());
        fallback
    }
}
