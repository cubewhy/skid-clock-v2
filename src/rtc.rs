use chrono::{NaiveDateTime, TimeZone, Utc};
use esp_idf_svc::sys::{settimeofday, timeval};

use crate::rtc::ds1302::Ds1302;

pub mod ds1302;

pub trait UnifiedRtc {
    fn read_time(&mut self) -> anyhow::Result<NaiveDateTime>;
    fn set_time(&mut self, time: &NaiveDateTime) -> anyhow::Result<()>;
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
