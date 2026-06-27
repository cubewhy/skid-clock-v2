use anyhow::{Result, anyhow};
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};

pub struct SecretManager {
    nvs: EspNvs<NvsDefault>,
}

impl SecretManager {
    /// Initializes the SecretManager with a dedicated secure NVS namespace.
    pub fn new(partition: EspDefaultNvsPartition) -> Result<Self> {
        let nvs = EspNvs::new(partition, "secrets", true)
            .map_err(|e| anyhow!("Failed to initialize secure NVS namespace: {:?}", e))?;
        Ok(Self { nvs })
    }

    /// Derives a safe 15-character compliant NVS key from a key using an FNV-1a 32-bit hash.
    /// This bypasses the hardware limit where NVS keys cannot exceed 15 bytes.
    fn derive_key(key: &str) -> String {
        let mut hash: u32 = 2166136261;
        for byte in key.as_bytes() {
            hash ^= *byte as u32;
            hash = hash.wrapping_mul(16777619);
        }
        format!("{:08x}", hash)
    }

    /// Securely persists a Wi-Fi password linked to a specific key.
    pub fn save_password(&mut self, key: &str, password: &str) -> Result<()> {
        let key = Self::derive_key(key);
        self.nvs
            .set_str(&key, password)
            .map_err(|e| anyhow!("Failed to write secret to flash memory: {:?}", e))?;
        log::info!(
            "Securely persisted credentials slot for hash identifier: {}",
            key
        );
        Ok(())
    }

    /// Fetches a saved Wi-Fi password for a given key. Returns `Ok(None)` if no record exists.
    pub fn get_password(&self, key: &str) -> Result<Option<String>> {
        let key = Self::derive_key(key);
        // WPA2 Passwords max out at 63 characters; a 64-byte buffer is completely safe.
        let mut buf = [0u8; 64];

        match self.nvs.get_str(&key, &mut buf) {
            Ok(Some(password)) => Ok(Some(password.to_string())),
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow!(
                "Failed to retrieve data from storage vault: {:?}",
                e
            )),
        }
    }

    /// Purges saved credentials associated with a key from the storage partition.
    pub fn delete_password(&mut self, key: &str) -> Result<()> {
        let key = Self::derive_key(key);
        match self.nvs.remove(&key) {
            Ok(_) => {
                log::info!(
                    "Purged credentials associated with hash identifier: {}",
                    key
                );
                Ok(())
            }
            Err(e) => Err(anyhow!("Failed to clear target storage slot: {:?}", e)),
        }
    }
}
