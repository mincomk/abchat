use std::{collections::HashSet, sync::Arc};
use web_push::{
    ContentEncoding, IsahcWebPushClient, VapidSignatureBuilder, WebPushClient,
    WebPushMessageBuilder,
};

use crate::{AppResult, NotificationMode, Persistence, Subscription};

pub struct NotificationService {
    persistence: Arc<dyn Persistence>,
    vapid_private_key: String,
    client: IsahcWebPushClient,
}

impl NotificationService {
    pub fn new(
        persistence: Arc<dyn Persistence>,
        _vapid_public_key: String,
        vapid_private_key: String,
    ) -> Self {
        Self {
            persistence,
            vapid_private_key,
            client: IsahcWebPushClient::new().expect("Failed to create web-push client"),
        }
    }

    pub async fn send_targeted_notification(
        &self,
        title: &str,
        body: &str,
        modes: Vec<NotificationMode>,
        usernames: Vec<String>,
        exclude_username: Option<String>,
    ) -> AppResult<()> {
        let mut subscriptions = self.persistence.get_subscriptions_by_mode(modes).await?;

        for username in usernames {
            // Only add if not already in All/Targeted mode list and if they are in Critical mode
            // Actually, we can just fetch their subscriptions and let deduplication handle it.
            // But checking mode first is better for performance.
            let mode = self
                .persistence
                .get_user_notification_mode(&username)
                .await?;
            if mode == NotificationMode::Critical {
                let user_subs = self.persistence.list_subscriptions(&username).await?;
                subscriptions.extend(user_subs);
            }
        }

        // Deduplicate and filter exclude_username
        let mut unique_endpoints = HashSet::new();
        let mut filtered_subs = Vec::new();

        for sub in subscriptions {
            if let Some(ref exclude) = exclude_username {
                if sub.username == *exclude {
                    continue;
                }
            }

            if unique_endpoints.insert(sub.endpoint.clone()) {
                filtered_subs.push(sub);
            }
        }

        let payload = serde_json::json!({
            "title": title,
            "body": body,
        })
        .to_string();

        for sub in filtered_subs {
            if let Err(e) = self.send_notification(&sub, &payload).await {
                tracing::error!("Failed to send notification to {}: {:?}", sub.username, e);
                if e.to_string().contains("410") || e.to_string().contains("404") {
                    let _ = self.persistence.delete_subscription(&sub.endpoint).await;
                }
            }
        }

        Ok(())
    }

    pub async fn send_to_all_relevant(
        &self,
        title: &str,
        body: &str,
        is_critical: bool,
    ) -> AppResult<()> {
        let modes = if is_critical {
            vec![NotificationMode::All, NotificationMode::Critical]
        } else {
            vec![NotificationMode::All]
        };

        let subscriptions = self.persistence.get_subscriptions_by_mode(modes).await?;

        let payload = serde_json::json!({
            "title": title,
            "body": body,
        })
        .to_string();

        for sub in subscriptions {
            if let Err(e) = self.send_notification(&sub, &payload).await {
                tracing::error!("Failed to send notification to {}: {:?}", sub.username, e);
                // Handle invalid subscriptions (cleanup) if it's a 404 or 410
                if e.to_string().contains("410") || e.to_string().contains("404") {
                    let _ = self.persistence.delete_subscription(&sub.endpoint).await;
                }
            }
        }

        Ok(())
    }

    async fn send_notification(&self, sub: &Subscription, payload: &str) -> AppResult<()> {
        let subscription_info =
            web_push::SubscriptionInfo::new(&sub.endpoint, &sub.p256dh, &sub.auth);

        let mut builder = WebPushMessageBuilder::new(&subscription_info);
        builder.set_payload(ContentEncoding::Aes128Gcm, payload.as_bytes());

        // web-push 0.11.0 VapidSignatureBuilder::from_base64 takes the private key
        let sig_builder =
            VapidSignatureBuilder::from_base64(&self.vapid_private_key, &subscription_info)
                .map_err(|e: web_push::WebPushError| {
                    crate::AppError::Service(crate::ServiceError::Internal(e.to_string()))
                })?;

        let signature = sig_builder.build().map_err(|e: web_push::WebPushError| {
            crate::AppError::Service(crate::ServiceError::Internal(e.to_string()))
        })?;

        builder.set_vapid_signature(signature);

        let message = builder.build().map_err(|e: web_push::WebPushError| {
            crate::AppError::Service(crate::ServiceError::Internal(e.to_string()))
        })?;

        self.client
            .send(message)
            .await
            .map_err(|e: web_push::WebPushError| {
                crate::AppError::Service(crate::ServiceError::Internal(e.to_string()))
            })?;

        Ok(())
    }
}
