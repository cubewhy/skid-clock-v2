use crate::display::UnifiedDisplay;
use crate::network_manager::{NetState, NetworkController};
use crate::rtc::sync_system_to_rtc;
use crate::ui::widgets::keyboard::KeyboardState;
use crate::{
    app_context::AppContext,
    app_context::UpdateContext,
    apps::App,
    ui::{
        Rect, Ui, UiEvents,
        layout::{FlexDirection, FlexNode},
    },
};
use std::vec::Vec;

use esp_idf_svc::sntp::{SntpConf, SyncStatus};
use esp_idf_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, WifiDeviceId};

pub struct NetworkSettingsState {
    tick: u32,
    net_state: NetState,
    last_global_state: Option<NetState>,
    menu_index: usize,
    scan_list: Vec<(String, AuthMethod)>,
    selected_ssid: String,
    selected_auth: AuthMethod,
    kb_state: KeyboardState,
    last_input_time: std::time::Instant,
    connected_ssid: String,
    connected_ip: String,
    connected_mac: String,
}

impl NetworkSettingsState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            net_state: NetState::Idle,
            last_global_state: None,
            menu_index: 0,
            scan_list: Vec::new(),
            selected_ssid: String::new(),
            selected_auth: AuthMethod::WPA2Personal,
            kb_state: KeyboardState::new(32),
            last_input_time: std::time::Instant::now(),
            connected_ssid: String::from("Disconnected"),
            connected_ip: String::from("0.0.0.0"),
            connected_mac: String::from("00:00:00:00:00:00"),
        }
    }
}

impl Default for NetworkSettingsState {
    fn default() -> Self {
        Self::new()
    }
}

fn spawn_wifi_scan(controller: &NetworkController) {
    let wifi_arc = controller.wifi.clone();
    let state_arc = controller.state.clone();
    let results_arc = controller.scan_results.clone();

    std::thread::spawn(move || {
        if let Ok(mut lock) = state_arc.lock() {
            *lock = NetState::Scanning;
        }

        match (|| -> anyhow::Result<Vec<(String, AuthMethod)>> {
            let mut wifi_lock = wifi_arc
                .lock()
                .map_err(|_| anyhow::anyhow!("Wifi lock poisoned"))?;

            if let Err(e) = wifi_lock.start() {
                log::warn!("Wi-Fi interface start command flagged status: {:?}", e);
            }

            let scanned = wifi_lock.scan().inspect_err(|&e| {
                log::error!("Wi-Fi low-level driver scan sequence aborted: {:?}", e);
            })?;

            let mut res = Vec::new();
            for ap in scanned {
                if !ap.ssid.is_empty()
                    && !res
                        .iter()
                        .any(|(s, _): &(String, AuthMethod)| **s == *ap.ssid)
                {
                    res.push((
                        ap.ssid.to_string(),
                        ap.auth_method.unwrap_or(AuthMethod::None),
                    ));
                }
            }
            Ok(res)
        })() {
            Ok(list) => {
                if let Ok(mut res_lock) = results_arc.lock() {
                    *res_lock = list;
                }
                if let Ok(mut state_lock) = state_arc.lock() {
                    *state_lock = NetState::SelectNetwork;
                }
            }
            Err(_) => {
                if let Ok(mut state_lock) = state_arc.lock() {
                    *state_lock = NetState::Error("Scan Failed");
                }
            }
        }
    });
}

fn spawn_wifi_connect(
    controller: &NetworkController,
    ssid: String,
    password: Option<String>,
    auth_method: AuthMethod,
) {
    let wifi_arc = controller.wifi.clone();
    let state_arc = controller.state.clone();
    let secret_manager_arc = controller.secret_manager.clone(); // Clone handle

    std::thread::spawn(move || {
        if let Ok(mut lock) = state_arc.lock() {
            *lock = NetState::Connecting;
        }

        let ssid_clone = ssid.clone();
        let password_clone = password.clone();

        match (|| -> anyhow::Result<()> {
            {
                let mut wifi_lock = wifi_arc
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Wifi lock poisoned"))?;

                let mut client_config = ClientConfiguration {
                    ssid: ssid.as_str().try_into().unwrap_or_default(),
                    auth_method,
                    ..Default::default()
                };

                if let Some(ref pwd) = password {
                    client_config.password = pwd.as_str().try_into().unwrap_or_default();
                } else {
                    client_config.password = "".try_into().unwrap_or_default();
                }

                wifi_lock.set_configuration(&Configuration::Client(client_config))?;
                wifi_lock.start()?;
                wifi_lock.connect()?;
            }

            let mut retry = 0;
            let mut has_valid_ip = false;

            while retry < 40 {
                std::thread::sleep(std::time::Duration::from_millis(250));

                if let Ok(wifi_lock) = wifi_arc.lock()
                    && let Ok(true) = wifi_lock.is_connected()
                    && let Ok(ip_info) = wifi_lock.sta_netif().get_ip_info()
                    && !ip_info.ip.is_unspecified()
                {
                    has_valid_ip = true;
                    break;
                }
                retry += 1;
            }

            if !has_valid_ip {
                return Err(anyhow::anyhow!("DHCP IP lease assignment timeout"));
            }
            Ok(())
        })() {
            Ok(_) => {
                if let Ok(mut lock) = state_arc.lock() {
                    *lock = NetState::Connected;
                }
                // Save password on successful verification
                if let Some(pwd) = password_clone
                    && let Ok(mut sm) = secret_manager_arc.lock()
                    && let Err(e) = sm.save_password(&ssid_clone, &pwd)
                {
                    log::error!("Failed to save verified password: {:?}", e);
                }
            }
            Err(_) => {
                if let Ok(mut lock) = state_arc.lock() {
                    *lock = NetState::Error("Conn Failed");
                }
                // Wipe cache record on failure to handle out-of-sync credentials gracefully
                if let Ok(mut sm) = secret_manager_arc.lock() {
                    let _ = sm.delete_password(&ssid_clone);
                }
            }
        }
    });
}

fn spawn_wifi_disconnect(controller: &NetworkController) {
    let wifi_arc = controller.wifi.clone();
    let state_arc = controller.state.clone();

    std::thread::spawn(move || {
        match (|| -> anyhow::Result<()> {
            let mut wifi_lock = wifi_arc
                .lock()
                .map_err(|_| anyhow::anyhow!("Wifi lock poisoned"))?;
            wifi_lock.disconnect()?;
            wifi_lock.stop()?;
            Ok(())
        })() {
            Ok(_) => {
                if let Ok(mut lock) = state_arc.lock() {
                    *lock = NetState::Idle;
                }
            }
            Err(_) => {
                if let Ok(mut lock) = state_arc.lock() {
                    *lock = NetState::Error("Disconnect Failed");
                }
            }
        }
    });
}

fn spawn_ntp_sync(controller: &NetworkController) {
    let sntp_arc = controller.sntp.clone();
    let state_arc = controller.state.clone();

    std::thread::spawn(move || {
        if let Ok(mut lock) = state_arc.lock() {
            *lock = NetState::NtpSyncing;
        }

        if let Ok(mut sntp_lock) = sntp_arc.lock()
            && sntp_lock.is_none()
        {
            let mut config = SntpConf::default();
            if !config.servers.is_empty() {
                config.servers[0] = "0.cn.pool.ntp.org";
            }

            match esp_idf_svc::sntp::EspSntp::new(&config) {
                Ok(sntp_instance) => {
                    *sntp_lock = Some(sntp_instance);
                    unsafe {
                        esp_idf_svc::sys::sntp_set_sync_mode(
                            esp_idf_svc::sys::sntp_sync_mode_t_SNTP_SYNC_MODE_IMMED,
                        );
                    }
                }
                Err(e) => {
                    log::error!("CRITICAL: SNTP engine constructor failed: {:?}", e);
                    if let Ok(mut lock) = state_arc.lock() {
                        *lock = NetState::Error("NTP Init Failed");
                    }
                    return;
                }
            }
        }

        unsafe {
            esp_idf_svc::sys::sntp_set_sync_status(
                esp_idf_svc::sys::sntp_sync_status_t_SNTP_SYNC_STATUS_RESET,
            );
        }

        let mut retry = 0;
        let mut sync_success = false;

        while retry < 30 {
            if SyncStatus::from(unsafe { esp_idf_svc::sys::sntp_get_sync_status() })
                == SyncStatus::Completed
            {
                sync_success = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
            retry += 1;
        }

        if sync_success {
            log::info!("NTP Synchronization Success!");
            if let Ok(mut lock) = state_arc.lock() {
                *lock = NetState::NtpSuccess;
            }
        } else {
            log::error!("NTP telemetry update failed to synchronize inside deadline.");
            if let Ok(mut sntp_lock) = sntp_arc.lock() {
                *sntp_lock = None;
            }
            if let Ok(mut lock) = state_arc.lock() {
                *lock = NetState::Error("NTP Timeout");
            }
        }
    });
}

pub fn update(ctx: &mut UpdateContext, state: &mut NetworkSettingsState) -> Option<App> {
    state.tick += 1;
    let events = ctx.menu_events;

    // Fetch and sync runtime network telemetry asynchronously
    if let Ok(wifi_lock) = ctx.network.wifi.try_lock()
        && let Ok(connected) = wifi_lock.is_connected()
    {
        if connected {
            if let Ok(Configuration::Client(client_cfg)) = wifi_lock.get_configuration() {
                state.connected_ssid = client_cfg.ssid.to_string();
            }

            if let Ok(ip_info) = wifi_lock.sta_netif().get_ip_info() {
                if ip_info.ip.is_unspecified() {
                    state.connected_ip = String::from("Allocating...");
                } else {
                    state.connected_ip = ip_info.ip.to_string();
                }
            }
        } else {
            state.connected_ssid = String::from("Disconnected");
            state.connected_ip = String::from("0.0.0.0");
        }

        if let Ok(mac) = wifi_lock.get_mac(WifiDeviceId::Sta) {
            state.connected_mac = format!(
                "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
            );
        }
    }

    // Edge-triggered synchronization with the global controller status
    if let Ok(global_state) = ctx.network.state.lock() {
        if state.last_global_state.is_none() {
            state.last_global_state = Some(global_state.clone());
        } else if Some(&*global_state) != state.last_global_state.as_ref() {
            let next_global = global_state.clone();
            state.last_global_state = Some(next_global.clone());

            match &next_global {
                NetState::Scanning
                | NetState::Connecting
                | NetState::NtpSyncing
                | NetState::Connected
                | NetState::NtpSuccess
                | NetState::Error(_) => {
                    state.net_state = next_global;
                }
                NetState::SelectNetwork => {
                    state.net_state = next_global;
                    if let Ok(results) = ctx.network.scan_results.lock() {
                        state.scan_list = results.clone();
                    }
                    state.menu_index = 0;
                }
                NetState::Idle => {
                    state.net_state = NetState::Idle;
                    state.menu_index = 0;
                }
                _ => {}
            }
        }
    }

    // Keyboard Text Input State Processing
    if matches!(
        state.net_state,
        NetState::InputSSID | NetState::InputPassword
    ) {
        if events.contains(UiEvents::KEY_ESC) {
            let fallback_state = ctx.network.reset_state();
            state.net_state = NetState::Idle;
            state.last_global_state = Some(fallback_state);
            state.menu_index = 0;
            return None;
        }

        if state.last_input_time.elapsed() > std::time::Duration::from_millis(200) {
            let old_len = state.kb_state.text.len();
            state.kb_state.handle_event(events);

            if state.kb_state.text.len() != old_len || state.kb_state.confirmed {
                state.last_input_time = std::time::Instant::now();
            }
        }

        if state.kb_state.confirmed {
            state.kb_state.confirmed = false;
            let entered_text = std::mem::take(&mut state.kb_state.text);
            if state.net_state == NetState::InputSSID {
                state.selected_ssid = entered_text;

                // For manual hidden networks, check if we already have a saved password
                let saved_pwd = if let Ok(sm) = ctx.network.secret_manager.lock() {
                    sm.get_password(&state.selected_ssid).unwrap_or(None)
                } else {
                    None
                };

                if let Some(pwd) = saved_pwd {
                    spawn_wifi_connect(
                        ctx.network,
                        state.selected_ssid.clone(),
                        Some(pwd),
                        state.selected_auth,
                    );
                } else {
                    state.net_state = NetState::InputPassword;
                    ctx.network.set_state(NetState::InputPassword);
                    state.last_global_state = Some(NetState::InputPassword);
                    state.kb_state = KeyboardState::new(64);
                }
            } else {
                let pwd = if entered_text.is_empty() {
                    None
                } else {
                    Some(entered_text)
                };
                spawn_wifi_connect(
                    ctx.network,
                    state.selected_ssid.clone(),
                    pwd,
                    state.selected_auth,
                );
            }
        }
        return None;
    }

    // Decoupled back/exit navigation
    if events.intersects(UiEvents::KEY_ESC | UiEvents::LEFT | UiEvents::KEY_4) {
        match state.net_state {
            NetState::Idle => {
                return Some(App::settings_menu());
            }
            NetState::ConfirmConnect => {
                state.net_state = NetState::SelectNetwork;
                state.menu_index = 0;
                return None;
            }
            _ => {
                state.net_state = NetState::Idle;
                state.menu_index = 0;

                let is_error_active = if let Ok(global_state) = ctx.network.state.lock() {
                    matches!(*global_state, NetState::Error(_))
                } else {
                    false
                };

                let fallback_state = if is_error_active {
                    ctx.network.reset_state()
                } else if let Ok(global_lock) = ctx.network.state.lock() {
                    global_lock.clone()
                } else {
                    NetState::Idle
                };

                state.last_global_state = Some(fallback_state);
                return None;
            }
        }
    }

    match &state.net_state {
        NetState::Idle => {
            let menu_strings = [
                "Scan Networks",
                "Hidden Network",
                "Disconnect Wi-Fi",
                "Manual NTP Sync",
            ];
            if events.intersects(UiEvents::UP | UiEvents::KEY_6) {
                state.menu_index = if state.menu_index == 0 {
                    menu_strings.len() - 1
                } else {
                    state.menu_index - 1
                };
            }
            if events.intersects(UiEvents::DOWN | UiEvents::KEY_5) {
                state.menu_index = (state.menu_index + 1) % menu_strings.len();
            }
            if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7 | UiEvents::RIGHT) {
                match state.menu_index {
                    0 => spawn_wifi_scan(ctx.network),
                    1 => {
                        state.net_state = NetState::InputSSID;
                        ctx.network.set_state(NetState::InputSSID);
                        state.last_global_state = Some(NetState::InputSSID);
                        state.kb_state = KeyboardState::new(32);
                        state.selected_auth = AuthMethod::WPA2Personal;
                    }
                    2 => spawn_wifi_disconnect(ctx.network),
                    3 => {
                        if ctx.network.is_connected() {
                            spawn_ntp_sync(ctx.network);
                        } else {
                            log::error!("Manual NTP Sync selected while offline.");
                            state.net_state = NetState::Error("WiFi Disconnected");
                            ctx.network.set_state(NetState::Error("WiFi Disconnected"));
                            state.last_global_state = Some(NetState::Error("WiFi Disconnected"));
                        }
                    }
                    _ => {}
                }
            }
        }

        NetState::SelectNetwork => {
            let total_items = state.scan_list.len() + 1;
            if events.intersects(UiEvents::UP | UiEvents::KEY_6) {
                state.menu_index = if state.menu_index == 0 {
                    total_items - 1
                } else {
                    state.menu_index - 1
                };
            }
            if events.intersects(UiEvents::DOWN | UiEvents::KEY_5) {
                state.menu_index = (state.menu_index + 1) % total_items;
            }
            if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7 | UiEvents::RIGHT) {
                if state.menu_index == total_items - 1 {
                    let fallback_state = ctx.network.reset_state();
                    state.net_state = NetState::Idle;
                    state.last_global_state = Some(fallback_state);
                    state.menu_index = 0;
                } else {
                    let (ssid, auth) = &state.scan_list[state.menu_index];
                    state.selected_ssid = ssid.clone();
                    state.selected_auth = *auth;

                    if *auth == AuthMethod::None {
                        spawn_wifi_connect(ctx.network, state.selected_ssid.clone(), None, *auth);
                    } else {
                        let has_saved = if let Ok(sm) = ctx.network.secret_manager.lock() {
                            sm.get_password(ssid).unwrap_or(None).is_some()
                        } else {
                            false
                        };

                        if has_saved {
                            state.net_state = NetState::ConfirmConnect;
                            ctx.network.set_state(NetState::ConfirmConnect);
                            state.last_global_state = Some(NetState::ConfirmConnect);
                            state.menu_index = 0;
                        } else {
                            state.net_state = NetState::InputPassword;
                            ctx.network.set_state(NetState::InputPassword);
                            state.last_global_state = Some(NetState::InputPassword);
                            state.kb_state = KeyboardState::new(64);
                        }
                    }
                }
            }
        }

        NetState::ConfirmConnect => {
            if events.intersects(UiEvents::UP | UiEvents::KEY_6) {
                state.menu_index = if state.menu_index == 0 { 1 } else { 0 };
            }
            if events.intersects(UiEvents::DOWN | UiEvents::KEY_5) {
                state.menu_index = (state.menu_index + 1) % 2;
            }
            if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7 | UiEvents::RIGHT) {
                match state.menu_index {
                    0 => {
                        let saved_pwd = if let Ok(sm) = ctx.network.secret_manager.lock() {
                            sm.get_password(&state.selected_ssid).unwrap_or(None)
                        } else {
                            None
                        };
                        spawn_wifi_connect(
                            ctx.network,
                            state.selected_ssid.clone(),
                            saved_pwd,
                            state.selected_auth,
                        );
                    }
                    1 => {
                        if let Ok(mut sm) = ctx.network.secret_manager.lock() {
                            let _ = sm.delete_password(&state.selected_ssid);
                        }
                        state.net_state = NetState::SelectNetwork;
                        ctx.network.set_state(NetState::SelectNetwork);
                        state.last_global_state = Some(NetState::SelectNetwork);
                        state.menu_index = 0;
                    }
                    _ => {}
                }
            }
        }
        NetState::Connected => {
            if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7) {
                spawn_ntp_sync(ctx.network);
            }
        }
        NetState::NtpSuccess => {
            if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7) {
                let _ = sync_system_to_rtc(ctx.rtc);
                return Some(App::main_menu());
            }
        }
        NetState::Error(_) if events.intersects(UiEvents::CONFIRM | UiEvents::KEY_7) => {
            let fallback_state = ctx.network.reset_state();
            state.net_state = NetState::Idle;
            state.last_global_state = Some(fallback_state);
            state.menu_index = 0;

            state.selected_ssid.clear();
            state.kb_state = KeyboardState::new(32);
        }
        _ => {}
    }
    None
}

pub fn draw(ctx: &mut AppContext, state: &NetworkSettingsState) {
    // Secondary Display Layout (0.96")
    let sub_bounds = ctx.display_0_96.rect();
    let mut sub_ui = Ui::new(&mut ctx.display_0_96, ctx.font);

    let mut sub_header_rect = Rect::default();
    let mut sub_divider_rect = Rect::default();
    let mut sub_body_rect = Rect::default();

    FlexNode::new(FlexDirection::Column)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(sub_bounds.width, 14)
                .assign_to(&mut sub_header_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(sub_bounds.width, 2)
                .assign_to(&mut sub_divider_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut sub_body_rect),
        )
        .layout(sub_bounds);

    sub_ui
        .label(sub_header_rect, "NETWORK STATUS")
        .center()
        .draw();
    sub_ui.horizontal_divider(sub_divider_rect);

    let mut ssid_rect = Rect::default();
    let mut ip_rect = Rect::default();
    let mut mac_rect = Rect::default();

    FlexNode::new(FlexDirection::Column)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut ssid_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut ip_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut mac_rect),
        )
        .layout(sub_body_rect);

    sub_ui
        .label(ssid_rect, &format!("SSID: {}", state.connected_ssid))
        .draw();
    sub_ui
        .label(ip_rect, &format!("IP:   {}", state.connected_ip))
        .draw();
    sub_ui
        .label(mac_rect, &format!("MAC:  {}", state.connected_mac))
        .scroll(state.tick, 2)
        .draw();

    // Primary Display Layout (1.3")
    let display_bounds = ctx.display_1_3.rect();
    let mut ui = Ui::new(&mut ctx.display_1_3, ctx.font);

    if state.net_state == NetState::InputSSID || state.net_state == NetState::InputPassword {
        let title = if state.net_state == NetState::InputSSID {
            "ENTER SSID"
        } else {
            "ENTER PASSWORD"
        };
        ui.keyboard(display_bounds, &state.kb_state, title);
        return;
    }

    let mut header_rect = Rect::default();
    let mut divider_rect = Rect::default();
    let mut body_rect = Rect::default();
    FlexNode::new(FlexDirection::Column)
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 14)
                .assign_to(&mut header_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_size(display_bounds.width, 2)
                .assign_to(&mut divider_rect),
        )
        .child(
            FlexNode::new(FlexDirection::Row)
                .with_flex(1)
                .assign_to(&mut body_rect),
        )
        .layout(display_bounds);

    ui.label(header_rect, "WIFI SETTINGS").center().draw();
    ui.horizontal_divider(divider_rect);

    match &state.net_state {
        NetState::Idle => {
            let menu_strings = [
                "Scan Networks",
                "Hidden Network",
                "Disconnect Wi-Fi",
                "Manual NTP Sync",
            ];
            ui.scroll_list(
                body_rect,
                &menu_strings,
                state.menu_index,
                3,
                12,
                |ui_ctx, r, text, sel| {
                    if sel {
                        ui_ctx.label(r, &format!("> {}", text)).draw();
                    } else {
                        ui_ctx.label(r, &format!("  {}", text)).draw();
                    }
                },
            );
        }
        NetState::Scanning => {
            ui.indeterminate_progress_bar(body_rect, "Scanning WiFi...", state.tick, 2);
        }
        NetState::SelectNetwork => {
            let mut list_items = Vec::new();
            for (ssid, auth) in &state.scan_list {
                let flag = if *auth == AuthMethod::None {
                    " [Open]"
                } else {
                    ""
                };
                list_items.push(format!("{}{}", ssid, flag));
            }
            list_items.push("<< Back to Menu".to_string());
            let refs: Vec<&str> = list_items.iter().map(|s| s.as_str()).collect();
            ui.scroll_list(
                body_rect,
                &refs,
                state.menu_index,
                4,
                12,
                |ui_ctx, r, text, sel| {
                    if sel {
                        ui_ctx.label(r, &format!("> {}", text)).draw();
                    } else {
                        ui_ctx.label(r, &format!("  {}", text)).draw();
                    }
                },
            );
        }
        NetState::ConfirmConnect => {
            let options = ["Connect Network", "Forget Network"];

            ui.label(
                body_rect.offset(0, 2),
                &format!("SSID: {}", state.selected_ssid),
            )
            .draw();

            ui.scroll_list(
                body_rect.offset(0, 16),
                &options,
                state.menu_index,
                2,
                12,
                |ui_ctx, r, text, sel| {
                    if sel {
                        ui_ctx.label(r, &format!("> {}", text)).draw();
                    } else {
                        ui_ctx.label(r, &format!("  {}", text)).draw();
                    }
                },
            );
        }
        NetState::Connecting => {
            ui.label(body_rect.offset(0, 6), "Connecting to:")
                .center()
                .draw();
            ui.label(body_rect.offset(0, 18), &state.selected_ssid)
                .center()
                .draw();
        }
        NetState::Connected => {
            ui.label(body_rect.offset(0, 4), "Wi-Fi Connected!")
                .center()
                .draw();
            ui.label(body_rect.offset(0, 16), "Press [OK] to Sync NTP")
                .center()
                .draw();
        }
        NetState::NtpSyncing => {
            ui.label(body_rect.offset(0, 10), "Syncing NTP Time...")
                .center()
                .draw();
        }
        NetState::NtpSuccess => {
            ui.label(body_rect.offset(0, 4), "NTP Synchronization")
                .center()
                .draw();
            ui.label(body_rect.offset(0, 16), "Success! OK to Save")
                .center()
                .draw();
        }
        NetState::Error(msg) => {
            ui.label(body_rect.offset(0, 4), "Error Occurred!")
                .center()
                .draw();
            ui.label(body_rect.offset(0, 16), msg).center().draw();
        }
        _ => {}
    }
}
