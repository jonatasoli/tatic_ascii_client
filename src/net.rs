use anyhow::Result;
use tatic_lib::{Action, GameState};
use reqwest::Client;
use serde_json::json;
use tracing::{debug, error, info};

#[derive(Clone)]
pub struct NetworkClient {
    base_url: String,
    client: Client,
}

impl NetworkClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    /// Conecta ao servidor e obtém/cria uma partida
    pub async fn connect(&self) -> Result<String, String> {
        info!("Conectando ao servidor: {}", self.base_url);
        
        // Lista partidas existentes
        let url = format!("{}/matches", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Erro de conexão: {}", e))?;
        
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Erro ao decodificar: {}", e))?;
        
        // Pega primeira partida ou cria nova
        if let Some(matches) = data["data"].as_array() {
            if !matches.is_empty() {
                if let Some(match_id) = matches[0]["id"].as_str() {
                    info!("Usando partida existente: {}", match_id);
                    return Ok(match_id.to_string());
                }
            }
        }
        
        // Cria nova partida
        self.create_match("player1", "ai").await
    }

    /// Cria nova partida
    pub async fn create_match(&self, player1: &str, player2: &str) -> Result<String, String> {
        let url = format!("{}/match/create", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&json!({
                "player1": player1,
                "player2": player2
            }))
            .send()
            .await
            .map_err(|e| format!("Erro ao criar partida: {}", e))?;
        
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Erro ao decodificar: {}", e))?;
        
        data["data"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "ID da partida não retornado".to_string())
    }

    /// Obtém estado atual da partida
    pub async fn get_state(&self, match_id: &str) -> Result<GameState, String> {
        let url = format!("{}/state?match_id={}", self.base_url, match_id);
        
        debug!("GET {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Erro ao obter estado: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Erro HTTP: {}", response.status()));
        }
        
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Erro ao decodificar: {}", e))?;
        
        serde_json::from_value(data["data"].clone())
            .map_err(|e| format!("Erro ao parsear estado: {}", e))
    }

    /// Envia ação para o servidor
    pub async fn send_action(
        &self,
        match_id: &str,
        player_id: &str,
        action: Action,
    ) -> Result<GameState, String> {
        let url = format!("{}/action", self.base_url);
        
        let body = json!({
            "match_id": match_id,
            "player_id": player_id,
            "action": action
        });
        
        debug!("POST {} - Body: {}", url, body);
        
        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Erro ao enviar ação: {}", e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Erro HTTP: {}", error_text));
        }
        
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Erro ao decodificar: {}", e))?;
        
        if !data["success"].as_bool().unwrap_or(false) {
            return Err(data["error"].as_str().unwrap_or("Erro desconhecido").to_string());
        }
        
        serde_json::from_value(data["data"].clone())
            .map_err(|e| format!("Erro ao parsear estado: {}", e))
    }

    /// Solicita ação da IA
    pub async fn get_ai_action(&self, match_id: &str, ai_player: &str) -> Result<Action, String> {
        let url = format!("{}/ai/action", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&json!({
                "match_id": match_id,
                "ai_player": ai_player
            }))
            .send()
            .await
            .map_err(|e| format!("Erro ao solicitar IA: {}", e))?;
        
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Erro ao decodificar: {}", e))?;
        
        serde_json::from_value(data["data"].clone())
            .map_err(|e| format!("Erro ao parsear ação: {}", e))
    }
}
