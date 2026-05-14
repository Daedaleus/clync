use std::io::Cursor;
use std::sync::Arc;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use surrealdb::{Surreal, engine::remote::ws::Client};
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, SubscriptionKeys,
    VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

use crate::dto::session::SessionResponse;
use crate::error::AppError;
use crate::repository::{
    group::GroupRepository,
    push_subscription::PushSubscriptionRepository,
    traits::{GroupRepo, PushSubscriptionRepo},
};

pub struct PushService {
    push_repo: PushSubscriptionRepository,
    group_repo: GroupRepository,
    client: IsahcWebPushClient,
    /// Raw 32-byte P-256 private key, base64url encoded.
    vapid_private_key: String,
    #[allow(dead_code)]
    vapid_subject: String,
}

impl PushService {
    pub fn new(
        db: Arc<Surreal<Client>>,
        vapid_private_key: String,
        vapid_subject: String,
    ) -> Result<Self, web_push::WebPushError> {
        Ok(Self {
            push_repo: PushSubscriptionRepository::new(Arc::clone(&db)),
            group_repo: GroupRepository::new(db),
            client: IsahcWebPushClient::new()?,
            vapid_private_key,
            vapid_subject,
        })
    }

    /// Converts a raw 32-byte P-256 private key (base64url) to SEC1 DER.
    /// web-push uses OpenSSL's EcKey::private_key_from_der which expects SEC1,
    /// not PKCS8. SEC1 is the ECPrivateKey format (RFC 5915).
    fn key_to_sec1_der(b64url_key: &str) -> Option<Vec<u8>> {
        let raw = URL_SAFE_NO_PAD.decode(b64url_key).ok()?;
        if raw.len() != 32 {
            return None;
        }
        // SEC1 ECPrivateKey DER for P-256 (51 bytes total):
        //   SEQUENCE (49 bytes)
        //     INTEGER 1              (version)
        //     OCTET STRING [32 B]    (private key scalar)
        //     [0] EXPLICIT           (optional: curve OID P-256)
        //       OID 1.2.840.10045.3.1.7
        let mut der = vec![
            0x30, 0x31,                                                 // SEQUENCE (49 bytes)
            0x02, 0x01, 0x01,                                           // version = 1
            0x04, 0x20,                                                 // OCTET STRING (32 bytes)
        ];
        der.extend_from_slice(&raw);
        der.extend_from_slice(&[
            0xA0, 0x0A,                                                 // [0] EXPLICIT (10 bytes)
            0x06, 0x08, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x03, 0x01, 0x07, // OID P-256
        ]);
        Some(der)
    }

    /// Sends push notifications to subscribed members of the given groups.
    /// Silently skips if VAPID key is not configured.
    pub async fn notify_groups(
        &self,
        group_ids: &[String],
        session: &SessionResponse,
    ) -> Result<(), AppError> {
        tracing::info!("Push: session '{}' in groups {:?}", session.game, group_ids);

        if group_ids.is_empty() || self.vapid_private_key.is_empty() {
            tracing::info!("Push: skipping — no group_ids or VAPID key missing");
            return Ok(());
        }

        let sec1_der = match Self::key_to_sec1_der(&self.vapid_private_key) {
            Some(d) => d,
            None => {
                tracing::warn!("Invalid VAPID_PRIVATE_KEY — expected 32-byte P-256 scalar in base64url");
                return Ok(());
            }
        };

        // Collect unique member IDs across all groups, excluding the session creator
        let mut user_ids: Vec<String> = Vec::new();
        for group_id in group_ids {
            if let Some(group) = self.group_repo.find_by_id(group_id.clone()).await? {
                for member_id in group.members {
                    if member_id != session.user_id && !user_ids.contains(&member_id) {
                        user_ids.push(member_id);
                    }
                }
            }
        }

        tracing::info!("Push: notifying {} potential recipients", user_ids.len());

        if user_ids.is_empty() {
            tracing::info!("Push: no other members in group(s)");
            return Ok(());
        }

        let subscriptions = self.push_repo.find_by_user_ids(user_ids).await?;
        tracing::info!("Push: found {} subscription(s)", subscriptions.len());

        if subscriptions.is_empty() {
            tracing::info!("Push: no push subscriptions found for group members");
            return Ok(());
        }

        let url = session.group_ids.first()
            .map(|gid| format!("/groups/{gid}"))
            .unwrap_or_else(|| "/".into());

        let payload = serde_json::json!({
            "title": format!("WhatsUp – {}", session.game),
            "body": format!("{} möchte {} spielen", session.username, session.game),
            "url": url,
        });
        let payload_bytes = serde_json::to_vec(&payload).unwrap_or_default();

        for sub in subscriptions {
            let info = SubscriptionInfo {
                endpoint: sub.endpoint.clone(),
                keys: SubscriptionKeys { p256dh: sub.p256dh.clone(), auth: sub.auth.clone() },
            };

            let vapid = match VapidSignatureBuilder::from_der(Cursor::new(&sec1_der), &info) {
                Ok(v) => v,
                Err(e) => { tracing::error!("VAPID builder error: {e}"); continue; }
            };

            let sig = match vapid.build() {
                Ok(s) => s,
                Err(e) => { tracing::error!("VAPID sign error: {e}"); continue; }
            };

            let mut builder = WebPushMessageBuilder::new(&info);
            builder.set_payload(ContentEncoding::Aes128Gcm, &payload_bytes);
            builder.set_vapid_signature(sig);

            let msg = match builder.build() {
                Ok(m) => m,
                Err(e) => { tracing::error!("Push message build error: {e}"); continue; }
            };

            tracing::info!("Push: sending to {}…", &sub.endpoint[..40.min(sub.endpoint.len())]);
            if let Err(e) = self.client.send(msg).await {
                tracing::error!("Push send failed: {e}");
            } else {
                tracing::info!("Push: ✅ sent successfully");
            }
        }

        Ok(())
    }
}
