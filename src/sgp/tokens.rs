use crate::lcu::LcuClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementsToken {
    #[serde(rename = "accessToken", alias = "access_token")]
    pub access_token: String,
    #[serde(default)]
    pub entitlements: Vec<Value>,
    #[serde(default)]
    pub issuer: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SgpTokens {
    pub entitlements: Option<EntitlementsToken>,
    pub league_session_token: Option<String>,
}

impl SgpTokens {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_entitlements(&self) -> bool {
        self.entitlements.is_some()
    }

    pub fn has_league_session(&self) -> bool {
        self.league_session_token.is_some()
    }

    pub fn access_token(&self) -> Option<&str> {
        self.entitlements.as_ref().and_then(|e| {
            if !e.access_token.is_empty() {
                Some(e.access_token.as_str())
            } else {
                e.token.as_deref().filter(|s| !s.is_empty())
            }
        })
    }

    pub fn session_token(&self) -> Option<&str> {
        self.league_session_token.as_deref().filter(|s| !s.is_empty())
    }
}

pub async fn extract_entitlements_token(lcu: &LcuClient) -> Option<EntitlementsToken> {
    let val = lcu.get_entitlements_token().await?;
    serde_json::from_value::<EntitlementsToken>(val).ok()
}

pub async fn extract_league_session_token(lcu: &LcuClient) -> Option<String> {
    lcu.get_league_session_token().await
}

pub async fn extract_sgp_tokens(lcu: &LcuClient) -> SgpTokens {
    let entitlements = extract_entitlements_token(lcu).await;
    let league_session_token = extract_league_session_token(lcu).await;
    SgpTokens {
        entitlements,
        league_session_token,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_entitlements_token_deserialization_formats() {
        // 1. Standard camelCase
        let camel = json!({
            "accessToken": "tok_camel_123",
            "entitlements": [],
            "issuer": "riot",
            "subject": "user1"
        });
        let parsed_camel: EntitlementsToken = serde_json::from_value(camel).expect("Failed camelCase parse");
        assert_eq!(parsed_camel.access_token, "tok_camel_123");

        // 2. Snake case alias
        let snake = json!({
            "access_token": "tok_snake_456",
            "token": "backup_token"
        });
        let parsed_snake: EntitlementsToken = serde_json::from_value(snake).expect("Failed snake_case parse");
        assert_eq!(parsed_snake.access_token, "tok_snake_456");

        // 3. Fallback when access_token is empty string but token is present
        let fallback = json!({
            "accessToken": "",
            "token": "tok_fallback_789"
        });
        let parsed_fallback: EntitlementsToken = serde_json::from_value(fallback).expect("Failed fallback parse");
        let sgp_tokens = SgpTokens {
            entitlements: Some(parsed_fallback),
            league_session_token: Some("session_jwt_111".to_string()),
        };
        assert_eq!(sgp_tokens.access_token(), Some("tok_fallback_789"));
        assert_eq!(sgp_tokens.session_token(), Some("session_jwt_111"));
        assert!(sgp_tokens.has_entitlements());
        assert!(sgp_tokens.has_league_session());
    }
}

