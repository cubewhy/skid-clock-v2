use esp_idf_svc::eventloop::{EspSubscription, EspSystemEventLoop, System};
use esp_idf_svc::sntp::EspSntp;
use esp_idf_svc::wifi::{AuthMethod, EspWifi, WifiEvent};
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

/// Lightweight structure storing connection details to completely decouple
/// the UI polling loops from blocking hardware driver locks.
#[derive(Default, Clone, Debug)]
pub struct ConnectionCache {
    pub is_connected: bool,
    pub ssid: String,
    pub ip: String,
}

/// A globally shareable network interface controller
#[derive(Clone)]
pub struct NetworkController {
    pub wifi: Arc<Mutex<EspWifi<'static>>>,
    pub sntp: Arc<Mutex<Option<EspSntp<'static>>>>,
    pub state: Arc<Mutex<NetState>>,
    pub scan_results: Arc<Mutex<Vec<(String, AuthMethod)>>>,
    pub cache: Arc<Mutex<ConnectionCache>>,
    _subscription: Arc<EspSubscription<'static, System>>,
}

impl NetworkController {
    pub fn new(
        modem: esp_idf_svc::hal::modem::Modem<'static>,
        sys_loop: EspSystemEventLoop,
        nvs: esp_idf_svc::nvs::EspDefaultNvsPartition,
    ) -> anyhow::Result<Self> {
        let wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs)).inspect_err(|&e| {
            log::error!("Failed to initialize hardware EspWifi driver: {:?}", e);
        })?;

        let state = Arc::new(Mutex::new(NetState::Idle));
        let cache = Arc::new(Mutex::new(ConnectionCache::default()));

        let state_trigger = state.clone();
        let cache_trigger = cache.clone();

        // Subscribe to system Wi-Fi events to automatically handle disconnections
        let subscription = sys_loop.subscribe::<WifiEvent, _>(move |event| match event {
            WifiEvent::StaDisconnected(_) => {
                log::warn!("WiFi disconnected event captured from system event loop.");

                // Instantly wipe connection metadata cache cleanly on drop triggers
                if let Ok(mut cache_lock) = cache_trigger.lock() {
                    cache_lock.is_connected = false;
                    cache_lock.ssid.clear();
                    cache_lock.ip.clear();
                }

                if let Ok(mut lock) = state_trigger.lock()
                    && matches!(
                        *lock,
                        NetState::Connected | NetState::NtpSyncing | NetState::NtpSuccess
                    )
                {
                    *lock = NetState::Idle;
                }
            }
            WifiEvent::StaConnected(_) => {
                log::info!("WiFi connected event captured.");
            }
            _ => {}
        })?;

        Ok(Self {
            wifi: Arc::new(Mutex::new(wifi)),
            sntp: Arc::new(Mutex::new(None)),
            state,
            scan_results: Arc::new(Mutex::new(Vec::new())),
            cache,
            _subscription: Arc::new(subscription),
        })
    }

    /// Non-blocking evaluation utilizing the local cached state layer.
    pub fn is_connected(&self) -> bool {
        self.cache.lock().map(|c| c.is_connected).unwrap_or(false)
    }

    /// Fetches the active SSID directly from the cache safely without driver locks.
    pub fn get_connected_ssid(&self) -> Option<String> {
        let cache_lock = self.cache.lock().ok()?;
        if cache_lock.is_connected && !cache_lock.ssid.is_empty() {
            Some(cache_lock.ssid.clone())
        } else {
            None
        }
    }

    /// Fetches the assigned IP address directly from the cache safely without driver locks.
    pub fn get_ip_address(&self) -> Option<String> {
        let cache_lock = self.cache.lock().ok()?;
        if cache_lock.is_connected && !cache_lock.ip.is_empty() {
            Some(cache_lock.ip.clone())
        } else {
            None
        }
    }

    /// Fetches current RSSI using a non-blocking try_lock configuration.
    /// If the driver is occupied scanning, it returns None instantly instead of freezing frames.
    pub fn get_rssi(&self) -> Option<i32> {
        if !self.is_connected() {
            return None;
        }

        let _wifi_lock = self.wifi.try_lock().ok()?;

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

    /// Dynamically determines fallback states using non-blocking primitives
    pub fn reset_state(&self) -> NetState {
        let fallback = if self.is_connected() {
            NetState::Connected
        } else {
            NetState::Idle
        };

        self.set_state(fallback.clone());
        fallback
    }
}
