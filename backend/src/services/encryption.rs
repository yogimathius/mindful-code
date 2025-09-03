use crate::error::{AppError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

pub struct EncryptionService {
    cipher: Aes256Gcm,
    key_id: String,
    rotation_keys: HashMap<String, Aes256Gcm>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedData {
    pub data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub key_id: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivacySettings {
    pub data_retention_days: i32,
    pub analytics_enabled: bool,
    pub sharing_enabled: bool,
    pub encryption_level: EncryptionLevel,
    pub gdpr_compliant: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EncryptionLevel {
    Basic,
    Standard,
    High,
    Military,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            data_retention_days: 365,
            analytics_enabled: true,
            sharing_enabled: false,
            encryption_level: EncryptionLevel::Standard,
            gdpr_compliant: true,
        }
    }
}

impl EncryptionService {
    pub fn new(master_key: &[u8; 32]) -> Result<Self> {
        let key = Key::<Aes256Gcm>::from_slice(master_key);
        let cipher = Aes256Gcm::new(key);
        let key_id = format!("key_{}", chrono::Utc::now().timestamp());

        info!("✅ Encryption service initialized with AES-256-GCM");

        Ok(Self {
            cipher,
            key_id,
            rotation_keys: HashMap::new(),
        })
    }

    pub fn from_hex_key(hex_key: &str) -> Result<Self> {
        if hex_key.len() != 64 {
            return Err(AppError::Encryption(
                "Master key must be 32 bytes (64 hex characters)".to_string(),
            ));
        }

        let mut key_bytes = [0u8; 32];
        hex::decode_to_slice(hex_key, &mut key_bytes)
            .map_err(|e| AppError::Encryption(format!("Invalid hex key: {}", e)))?;

        Self::new(&key_bytes)
    }

    pub fn encrypt_sensitive_data<T>(&self, data: &T) -> Result<EncryptedData>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_vec(data)
            .map_err(|e| AppError::Encryption(format!("Serialization failed: {}", e)))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = self
            .cipher
            .encrypt(nonce, serialized.as_ref())
            .map_err(|e| AppError::Encryption(format!("Encryption failed: {}", e)))?;

        debug!("Encrypted {} bytes of sensitive data", serialized.len());

        Ok(EncryptedData {
            data: encrypted,
            nonce: nonce_bytes.to_vec(),
            key_id: self.key_id.clone(),
            created_at: chrono::Utc::now().timestamp_millis(),
        })
    }

    pub fn decrypt_sensitive_data<T>(&self, encrypted_data: &EncryptedData) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let cipher = if encrypted_data.key_id == self.key_id {
            &self.cipher
        } else {
            self.rotation_keys
                .get(&encrypted_data.key_id)
                .ok_or_else(|| {
                    AppError::Encryption(format!(
                        "Decryption key not found: {}",
                        encrypted_data.key_id
                    ))
                })?
        };

        let nonce = Nonce::from_slice(&encrypted_data.nonce);

        let decrypted = cipher
            .decrypt(nonce, encrypted_data.data.as_ref())
            .map_err(|e| AppError::Encryption(format!("Decryption failed: {}", e)))?;

        let data = serde_json::from_slice(&decrypted)
            .map_err(|e| AppError::Encryption(format!("Deserialization failed: {}", e)))?;

        debug!("Decrypted {} bytes of sensitive data", decrypted.len());

        Ok(data)
    }

    pub fn encrypt_field(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::Encryption(format!("Field encryption failed: {}", e)))?;

        let combined = [nonce_bytes.as_slice(), &encrypted].concat();
        Ok(base64::encode(combined))
    }

    pub fn decrypt_field(&self, encrypted_field: &str) -> Result<String> {
        let combined = base64::decode(encrypted_field)
            .map_err(|e| AppError::Encryption(format!("Base64 decode failed: {}", e)))?;

        if combined.len() < 12 {
            return Err(AppError::Encryption("Invalid encrypted field format".to_string()));
        }

        let (nonce_bytes, encrypted_data) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted = self
            .cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|e| AppError::Encryption(format!("Field decryption failed: {}", e)))?;

        String::from_utf8(decrypted)
            .map_err(|e| AppError::Encryption(format!("UTF-8 decode failed: {}", e)))
    }

    pub fn rotate_key(&mut self, new_master_key: &[u8; 32]) -> Result<String> {
        let old_key_id = self.key_id.clone();
        let old_cipher = std::mem::replace(
            &mut self.cipher,
            Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(new_master_key)),
        );

        // Store old key for decrypting existing data
        self.rotation_keys.insert(old_key_id.clone(), old_cipher);

        // Generate new key ID
        self.key_id = format!("key_{}", chrono::Utc::now().timestamp());

        info!("🔄 Encryption key rotated. Old key ID: {}", old_key_id);

        Ok(old_key_id)
    }

    pub fn generate_master_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    pub fn key_to_hex(key: &[u8; 32]) -> String {
        hex::encode(key)
    }

    pub async fn secure_delete_data(&self, data: &mut [u8]) {
        // Overwrite with random data multiple times for secure deletion
        for _ in 0..3 {
            OsRng.fill_bytes(data);
        }
        // Final overwrite with zeros
        data.fill(0);
    }

    pub fn get_current_key_id(&self) -> &str {
        &self.key_id
    }

    pub fn has_key(&self, key_id: &str) -> bool {
        key_id == self.key_id || self.rotation_keys.contains_key(key_id)
    }

    pub fn cleanup_old_keys(&mut self, max_age_days: i64) {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (max_age_days * 24 * 60 * 60);
        
        let keys_to_remove: Vec<String> = self
            .rotation_keys
            .keys()
            .filter(|key_id| {
                if let Some(timestamp_str) = key_id.strip_prefix("key_") {
                    if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                        timestamp < cutoff_timestamp
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        for key_id in keys_to_remove {
            self.rotation_keys.remove(&key_id);
            warn!("🗑️ Removed old encryption key: {}", key_id);
        }
    }
}

#[derive(Debug, Clone)]
pub struct GdprDataExport {
    pub user_id: uuid::Uuid,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub data_categories: Vec<DataCategory>,
    pub total_records: u32,
    pub format: ExportFormat,
}

#[derive(Debug, Clone)]
pub struct DataCategory {
    pub category: String,
    pub record_count: u32,
    pub encrypted: bool,
    pub retention_period_days: i32,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
}

pub struct PrivacyManager {
    encryption: EncryptionService,
}

impl PrivacyManager {
    pub fn new(encryption: EncryptionService) -> Self {
        Self { encryption }
    }

    pub async fn export_user_data(
        &self,
        db: &sqlx::PgPool,
        user_id: uuid::Uuid,
        format: ExportFormat,
    ) -> Result<GdprDataExport> {
        let mut data_categories = Vec::new();

        // Export coding sessions
        let sessions_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM coding_sessions WHERE user_id = $1",
            user_id
        )
        .fetch_one(db)
        .await?
        .unwrap_or(0) as u32;

        if sessions_count > 0 {
            data_categories.push(DataCategory {
                category: "coding_sessions".to_string(),
                record_count: sessions_count,
                encrypted: false,
                retention_period_days: 365,
            });
        }

        // Export flow states
        let flow_states_count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) 
            FROM flow_states fs
            JOIN coding_sessions cs ON fs.session_id = cs.id
            WHERE cs.user_id = $1
            "#,
            user_id
        )
        .fetch_one(db)
        .await?
        .unwrap_or(0) as u32;

        if flow_states_count > 0 {
            data_categories.push(DataCategory {
                category: "flow_states".to_string(),
                record_count: flow_states_count,
                encrypted: false,
                retention_period_days: 365,
            });
        }

        // Export encrypted data
        let encrypted_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM encrypted_user_data WHERE user_id = $1",
            user_id
        )
        .fetch_one(db)
        .await?
        .unwrap_or(0) as u32;

        if encrypted_count > 0 {
            data_categories.push(DataCategory {
                category: "encrypted_data".to_string(),
                record_count: encrypted_count,
                encrypted: true,
                retention_period_days: 2555, // 7 years for compliance
            });
        }

        let total_records = data_categories.iter().map(|c| c.record_count).sum();

        info!("📊 GDPR export prepared for user {}: {} records", user_id, total_records);

        Ok(GdprDataExport {
            user_id,
            exported_at: chrono::Utc::now(),
            data_categories,
            total_records,
            format,
        })
    }

    pub async fn delete_user_data(
        &self,
        db: &sqlx::PgPool,
        user_id: uuid::Uuid,
    ) -> Result<u32> {
        let mut tx = db.begin().await?;
        let mut deleted_count = 0;

        // Delete user insights
        let insights_deleted = sqlx::query!(
            "DELETE FROM user_insights WHERE user_id = $1",
            user_id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        deleted_count += insights_deleted;

        // Delete encrypted data
        let encrypted_deleted = sqlx::query!(
            "DELETE FROM encrypted_user_data WHERE user_id = $1",
            user_id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        deleted_count += encrypted_deleted;

        // Delete flow states (via cascade from sessions)
        let sessions_deleted = sqlx::query!(
            "DELETE FROM coding_sessions WHERE user_id = $1",
            user_id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        deleted_count += sessions_deleted;

        // Delete team memberships
        let team_memberships_deleted = sqlx::query!(
            "DELETE FROM team_members WHERE user_id = $1",
            user_id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        deleted_count += team_memberships_deleted;

        // Finally delete user
        let user_deleted = sqlx::query!(
            "DELETE FROM users WHERE id = $1",
            user_id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        deleted_count += user_deleted;

        tx.commit().await?;

        info!("🗑️ GDPR deletion completed for user {}: {} records deleted", user_id, deleted_count);

        Ok(deleted_count as u32)
    }

    pub async fn anonymize_user_data(
        &self,
        db: &sqlx::PgPool,
        user_id: uuid::Uuid,
    ) -> Result<u32> {
        let mut tx = db.begin().await?;
        let mut anonymized_count = 0;

        // Anonymize user record
        let user_updated = sqlx::query!(
            r#"
            UPDATE users 
            SET email = 'anonymized_' || id::text || '@deleted.local',
                password_hash = 'ANONYMIZED',
                privacy_settings = '{}'::jsonb
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        anonymized_count += user_updated;

        // Anonymize session data
        let sessions_updated = sqlx::query!(
            r#"
            UPDATE coding_sessions 
            SET project_path = 'ANONYMIZED',
                environment_data = '{}'::jsonb
            WHERE user_id = $1
            "#,
            user_id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        anonymized_count += sessions_updated;

        tx.commit().await?;

        info!("🎭 Data anonymization completed for user {}: {} records anonymized", user_id, anonymized_count);

        Ok(anonymized_count as u32)
    }

    pub fn encrypt(&self, data: &impl Serialize) -> Result<EncryptedData> {
        self.encryption.encrypt_sensitive_data(data)
    }

    pub fn decrypt<T>(&self, encrypted: &EncryptedData) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.encryption.decrypt_sensitive_data(encrypted)
    }
}