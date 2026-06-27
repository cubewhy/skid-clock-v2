use chrono::{FixedOffset, NaiveDateTime, TimeZone, Utc};
use esp_idf_svc::sys::{settimeofday, timeval, tzset};

use crate::rtc::ds1302::Ds1302;

pub mod ds1302;

static TIMEZONE: i32 = 8;

pub trait UnifiedRtc {
    fn read_time(&mut self) -> anyhow::Result<NaiveDateTime>;
    fn set_time(&mut self, time: &NaiveDateTime) -> anyhow::Result<()>;
}

pub fn init_timezone() {
    unsafe {
        std::env::set_var("TZ", "CST-8");
        tzset();
    }
}

pub fn sync_time(rtc: &mut Ds1302) -> anyhow::Result<()> {
    let naive_dt = rtc.read_time()?;

    let timestamp_secs = if let Some(dt_utc) = chrono::Local.from_local_datetime(&naive_dt).single()
    {
        dt_utc.with_timezone(&Utc).timestamp()
    } else {
        naive_dt.and_utc().timestamp()
    };

    let tv = timeval {
        tv_sec: timestamp_secs as _,
        tv_usec: 0,
    };

    unsafe {
        if settimeofday(&tv, std::ptr::null()) == 0 {
            log::debug!("time sync completed: {tv:?}");
        } else {
            anyhow::bail!("failed to sync time to esp-idf");
        }
    }

    Ok(())
}

pub fn sync_rtc_to_system(rtc: &mut Ds1302) -> anyhow::Result<()> {
    let naive_dt = rtc.read_time()?;
    let offset = FixedOffset::east_opt(TIMEZONE * 3600).unwrap();

    if let Some(dt) = offset.from_local_datetime(&naive_dt).single() {
        let tv = timeval {
            tv_sec: dt.with_timezone(&Utc).timestamp() as _,
            tv_usec: 0,
        };
        unsafe {
            settimeofday(&tv, std::ptr::null());
        }
    }
    Ok(())
}

pub fn sync_system_to_rtc(rtc: &mut Ds1302) -> anyhow::Result<()> {
    let mut now_secs: esp_idf_svc::sys::time_t = 0;
    unsafe {
        esp_idf_svc::sys::time(&mut now_secs);
    }

    let offset = FixedOffset::east_opt(TIMEZONE * 3600).unwrap();
    if let Some(dt) = offset.timestamp_opt(now_secs as i64, 0).single() {
        let naive_local = dt.naive_local();

        rtc.set_time(&naive_local)?;
        log::info!("Network NTP time synced back to DS1302: {:?}", naive_local);
    }
    Ok(())
}
