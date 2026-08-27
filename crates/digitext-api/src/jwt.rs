// Copyright (c) 2026 DigiText. All rights reserved.
use anyhow::Context;
use chrono::{DateTime, Duration, Utc};
use digitext_core::{PermissionOverrides, Role};
use digitext_db::users::User;
use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
use ed25519_dalek::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey};
use ed25519_dalek::SigningKey;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub const C_3: i64 = 15;
pub const C_EL: i64 = 30;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Purpose {
    Access,
    OfflineLease,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct T2s {
    pub sub: Uuid,
    pub username: String,
    pub role: Role,
    pub project_id: Option<Uuid>,
    pub device_id: Option<Uuid>,
    pub tv: i32,
    pub purpose: Purpose,
    pub iat: i64,
    pub exp: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offline_expiry: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub perm_grants: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub perm_denies: Vec<String>,
}
pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
    pub public_pem: String,
}
impl JwtKeys {
    pub fn from_private_pem(r_4x: &str) -> anyhow::Result<Self> {
        let r_4y = SigningKey::from_pkcs8_pem(r_4x)
            .context("UEVNIGtob8OhIGvDvSBFZDI1NTE5IGtow7RuZyBo4bujcCBs4buHIChj4bqnbiBQS0NTIzgp")?;
        let public_pem = r_4y
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .context("a2jDtG5nIHh14bqldCDEkcaw4bujYyBwdWJsaWMga2V5IFBFTQ==")?;
        Ok(JwtKeys {
            encoding: EncodingKey::from_ed_pem(r_4x.as_bytes())
                .context("anNvbndlYnRva2VuIGtow7RuZyDEkeG7jWMgxJHGsOG7o2Mga2hvw6Ega8O9")?,
            decoding: DecodingKey::from_ed_pem(public_pem.as_bytes())
                .context("anNvbndlYnRva2VuIGtow7RuZyDEkeG7jWMgxJHGsOG7o2MgcHVibGljIGtleQ==")?,
            public_pem,
        })
    }
    pub fn generate_ephemeral() -> Self {
        let r_50 = SigningKey::generate(&mut rand::rngs::OsRng);
        let r_4z = r_50
            .to_pkcs8_pem(LineEnding::LF)
            .expect("eHXhuqV0IFBFTSBraG/DoSB04bqhbQ==");
        Self::from_private_pem(&r_4z).expect("a2hvw6EgdOG6oW0gaOG7o3AgbOG7hw==")
    }
    pub fn load_or_generate(path: &str) -> anyhow::Result<Self> {
        if std::path::Path::new(path).exists() {
            let pem = std::fs::read_to_string(path)
                .with_context(|| format!("a2jDtG5nIMSR4buNYyDEkcaw4bujYyBraG/DoSBrw70g{path}"))?;
            return Self::from_private_pem(&pem);
        }
        tracing::warn!(
            "Y2jGsGEgY8OzIGtob8OhIGvDvSA={path}IOKAlCB04buxIHNpbmggRWQyNTUxOSBt4bubaSAoZ2nhu68gZmlsZSBuw6B5IMSR4buDIHRva2VuIGtow7RuZyBi4buLIHbDtCBoaeG7h3Up"
        );
        let r_51 = SigningKey::generate(&mut rand::rngs::OsRng);
        let pem = r_51
            .to_pkcs8_pem(LineEnding::LF)
            .context("eHXhuqV0IFBFTSBraG/DoSBt4bubaQ==")?;
        std::fs::write(path, pem.as_bytes())
            .with_context(|| format!("a2jDtG5nIGdoaSDEkcaw4bujYyBraG/DoSBrw70g{path}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
        Self::from_private_pem(&pem)
    }
    fn sign(&self, r_52: &T2s) -> anyhow::Result<String> {
        jsonwebtoken::encode(&Header::new(Algorithm::EdDSA), r_52, &self.encoding)
            .context("a8O9IEpXVCB0aOG6pXQgYuG6oWk=")
    }
    pub fn sign_access(
        &self,
        user: &User,
        device_id: Option<Uuid>,
        r_53: DateTime<Utc>,
    ) -> anyhow::Result<String> {
        self.sign(&T2s {
            sub: user.id,
            username: user.username.clone(),
            role: user.role,
            project_id: user.project_id,
            device_id,
            tv: user.token_version,
            purpose: Purpose::Access,
            iat: r_53.timestamp(),
            exp: (r_53 + Duration::minutes(C_3)).timestamp(),
            offline_expiry: None,
            perm_grants: Vec::new(),
            perm_denies: Vec::new(),
        })
    }
    pub fn sign_offline_lease(
        &self,
        user: &User,
        r_57: &PermissionOverrides,
        device_id: Option<Uuid>,
        r_56: DateTime<Utc>,
    ) -> anyhow::Result<(String, DateTime<Utc>)> {
        let r_54 = r_56 + Duration::days(C_EL);
        let r_55 = self.sign(&T2s {
            sub: user.id,
            username: user.username.clone(),
            role: user.role,
            project_id: user.project_id,
            device_id,
            tv: user.token_version,
            purpose: Purpose::OfflineLease,
            iat: r_56.timestamp(),
            exp: r_54.timestamp(),
            offline_expiry: Some(r_54.timestamp()),
            perm_grants: r_57.grants.iter().map(|p| p.as_str().to_owned()).collect(),
            perm_denies: r_57.denies.iter().map(|p| p.as_str().to_owned()).collect(),
        })?;
        Ok((r_55, r_54))
    }
    pub fn decode(&self, token: &str) -> Result<T2s, jsonwebtoken::errors::Error> {
        let r_58 = Validation::new(Algorithm::EdDSA);
        Ok(jsonwebtoken::decode::<T2s>(token, &self.decoding, &r_58)?.claims)
    }
}
