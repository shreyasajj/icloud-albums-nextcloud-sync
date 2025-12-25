use anyhow::Result;
use rustpush::{ActivationInfo, OSConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Device configuration for iCloud authentication
/// Simulates a Mac device for authentication purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacDeviceConfig {
    pub device_uuid: String,
    pub device_name: String,
    pub device_model: String,
    pub serial_number: String,
    pub os_version: String,
    pub build_version: String,
}

impl Default for MacDeviceConfig {
    fn default() -> Self {
        Self {
            device_uuid: Uuid::new_v4().to_string().to_uppercase(),
            device_name: "iCloud Sync Server".to_string(),
            device_model: "MacBookPro18,1".to_string(),
            serial_number: Self::generate_serial(),
            os_version: "14.2.1".to_string(),
            build_version: "23C71".to_string(),
        }
    }
}

impl MacDeviceConfig {
    /// Generate a realistic-looking Mac serial number
    fn generate_serial() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Mac serial format: CCCYYWSSSS
        // CCC = Manufacturing location code
        // YY = Year (half-year)
        // W = Week of manufacture
        // SSSS = Unique identifier

        format!(
            "C02{}{}{}",
            // Year + week
            rng.gen_range(0..99),
            // Location
            (b'A' + rng.gen_range(0..26)) as char,
            // Unique ID
            format!("{:04X}", rng.gen_range(0..0x10000))
        )
    }

    /// Create from database values
    pub fn from_db(
        device_uuid: String,
        device_name: String,
        device_model: String,
    ) -> Self {
        Self {
            device_uuid,
            device_name,
            device_model,
            serial_number: Self::generate_serial(),
            os_version: "14.2.1".to_string(),
            build_version: "23C71".to_string(),
        }
    }

    /// Save device configuration to database
    pub async fn save_to_db(&self, pool: &sqlx::PgPool) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE config SET value = $1 WHERE key = 'device_uuid';
            UPDATE config SET value = $2 WHERE key = 'device_name';
            UPDATE config SET value = $3 WHERE key = 'device_model';
            "#
        )
        .bind(&self.device_uuid)
        .bind(&self.device_name)
        .bind(&self.device_model)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Load device configuration from database
    pub async fn load_from_db(pool: &sqlx::PgPool) -> Result<Option<Self>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            r#"
            SELECT key, value FROM config
            WHERE key IN ('device_uuid', 'device_name', 'device_model')
            "#
        )
        .fetch_all(pool)
        .await?;

        let mut config_map: HashMap<String, String> = rows.into_iter().collect();

        if config_map.is_empty() {
            return Ok(None);
        }

        let device_uuid = config_map.remove("device_uuid").unwrap_or_default();
        let device_name = config_map.remove("device_name").unwrap_or_default();
        let device_model = config_map.remove("device_model").unwrap_or_default();

        if device_uuid.is_empty() {
            return Ok(None);
        }

        Ok(Some(Self::from_db(device_uuid, device_name, device_model)))
    }
}

impl OSConfig for MacDeviceConfig {
    fn build_activation_info(&self, csr: Vec<u8>) -> ActivationInfo {
        ActivationInfo {
            activation_randomness: Uuid::new_v4().to_string(),
            activation_state: "Activated".to_string(),
            build_version: self.build_version.clone(),
            device_cert_request: csr,
            device_class: "MacOS".to_string(),
            product_type: self.device_model.clone(),
            product_version: self.os_version.clone(),
            serial_number: self.serial_number.clone(),
            unique_device_id: self.device_uuid.clone(),
        }
    }

    fn get_device_name(&self) -> String {
        self.device_name.clone()
    }

    fn get_device_uuid(&self) -> String {
        self.device_uuid.clone()
    }

    fn get_os_version(&self) -> String {
        self.os_version.clone()
    }

    fn get_software_name(&self) -> String {
        "macOS".to_string()
    }

    fn get_software_version(&self) -> String {
        self.os_version.clone()
    }

    fn get_software_build_id(&self) -> String {
        self.build_version.clone()
    }

    fn get_hardware_version(&self) -> String {
        self.device_model.clone()
    }

    fn get_product_type(&self) -> String {
        self.device_model.clone()
    }

    fn get_serial_number(&self) -> String {
        self.serial_number.clone()
    }
}

// Add rand to Cargo.toml
use rand;
