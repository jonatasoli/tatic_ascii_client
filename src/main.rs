//! Cliente GUI do RPG ASCII Tático
//!
//! Interface gráfica usando Iced 0.13.1 que:
//! - Renderiza o tabuleiro em ASCII
//! - Captura input do jogador (WASD/Enter ou mouse)
//! - Comunica com servidor via REST/WebSocket
//! - Loga todas as ações detalhadamente

use iced::{
    widget::{button, column, container, row, scrollable, text, Column, Container, Row, Text},
    Alignment, Application, Command, Element, Length, Settings, Theme,
};
use tracing::{error, info, warn};

mod input;
mod net;
mod renderer;
mod ui;

use tatic_lib::{Action, Coord, GameState};
use net::NetworkClient;
use renderer::BoardRenderer;

/// Estado principal da aplicação
#[derive(Debug)]
pub struct RpgAsciiClient {
    /// Cliente de rede
    network: NetworkClient,
    /// Estado atual do jogo
    game_state: Option<GameState>,
    /// ID da partida atual
    match_id: Option<String>,
    /// ID do jogador
    player_id: String,
    /// Coordenada selecionada (para movimento/ataque)
    selected_coord: Option<Coord>,
    /// Log de mensagens
    message_log: Vec<String>,
    /// Estado da conexão
    connection_status: ConnectionStatus,
    /// Modo de input
    input_mode: InputMode,
}

#[derive(Debug, Clone)]
enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone)]
enum InputMode {
    SelectUnit,
    SelectTarget,
    WaitingResponse,
}

/// Mensagens da aplicação
#[derive(Debug, Clone)]
pub enum Message {
    // Conexão
    Connect,
    ConnectionResult(Result<String, String>),
    
    // Ações do jogo
    CellClicked(Coord),
    SendAction(Action),
    ActionResult(Result<GameState, String>),
    
    // WebSocket
    WebSocketMessage(String),
    
    // UI
    ClearSelection,
    RefreshState,
    
    // Navegação
    KeyPressed(char),
    
    // IA
    RequestAiMove,
    AiMoveResult(Result<Action, String>),
}

impl Application for RpgAsciiClient {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        init_logging();
        
        info!("🎮 Iniciando cliente do RPG ASCII Tático");
        
        let client = Self {
            network: NetworkClient::new("http://localhost:3000".to_string()),
            game_state: None,
            match_id: None,
            player_id: "player1".to_string(),
            selected_coord: None,
            message_log: vec!["Bem-vindo ao RPG ASCII Tático!".to_string()],
            connection_status: ConnectionStatus::Disconnected,
            input_mode: InputMode::SelectUnit,
        };
        
        (client, Command::none())
    }

    fn title(&self) -> String {
        String::from("RPG ASCII Tático - Cliente")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Connect => {
                info!("🔌 Tentando conectar ao servidor...");
                self.connection_status = ConnectionStatus::Connecting;
                self.add_log("Conectando ao servidor...".to_string());
                
                let network = self.network.clone();
                Command::perform(
                    async move {
                        network.connect().await
                    },
                    Message::ConnectionResult,
                )
            }
            
            Message::ConnectionResult(result) => {
                match result {
                    Ok(match_id) => {
                        info!("✅ Conectado! Match ID: {}", match_id);
                        self.connection_status = ConnectionStatus::Connected;
                        self.match_id = Some(match_id.clone());
                        self.add_log(format!("Conectado à partida: {}", match_id));
                        
                        // Busca estado inicial
                        Command::perform(async {}, |_| Message::RefreshState)
                    }
                    Err(e) => {
                        error!("❌ Erro ao conectar: {}", e);
                        self.connection_status = ConnectionStatus::Error(e.clone());
                        self.add_log(format!("Erro: {}", e));
                        Command::none()
                    }
                }
            }
            
            Message::CellClicked(coord) => {
                info!("🖱️ Célula clicada: {}", coord);
                
                match self.input_mode {
                    InputMode::SelectUnit => {
                        // Verifica se há uma unidade do jogador
                        if let Some(state) = &self.game_state {
                            if let Some(cell) = state.board.get(&coord) {
                                if let Some(unit) = cell.get_unit() {
                                    if unit.owner == self.player_id {
                                        self.selected_coord = Some(coord);
                                        self.input_mode = InputMode::SelectTarget;
                                        self.add_log(format!("Unidade selecionada em {}", coord));
                                        info!("✅ Unidade selecionada: {}", coord);
                                    } else {
                                        self.add_log("Esta unidade não é sua!".to_string());
                                        warn!("⚠️ Tentou selecionar unidade inimiga");
                                    }
                                } else {
                                    self.add_log("Nenhuma unidade nesta posição".to_string());
                                }
                            }
                        }
                    }
                    
                    InputMode::SelectTarget => {
                        if let Some(from) = self.selected_coord {
                            // Determina se é movimento ou ataque
                            let action = if let Some(state) = &self.game_state {
                                if let Some(cell) = state.board.get(&coord) {
                                    if cell.get_unit().is_some() {
                                        Action::Attack { from, to: coord }
                                    } else {
                                        Action::Move { from, to: coord }
                                    }
                                } else {
                                    Action::Move { from, to: coord }
                                }
                            } else {
                                Action::Move { from, to: coord }
                            };
                            
                            self.input_mode = InputMode::WaitingResponse;
                            return Command::perform(async {}, move |_| Message::SendAction(action));
                        }
                    }
                    
                    InputMode::WaitingResponse => {
                        self.add_log("Aguardando resposta do servidor...".to_string());
                    }
                }
                
                Command::none()
            }
            
            Message::SendAction(action) => {
                if let Some(match_id) = &self.match_id {
                    info!("📤 Enviando ação: {:?}", action);
                    self.add_log(format!("Enviando: {:?}", action));
                    
                    let network = self.network.clone();
                    let match_id = match_id.clone();
                    let player_id = self.player_id.clone();
                    
                    // Log detalhado ANTES do request
                    info!(
                        "Client -> POST /action {{player: {}, action: {:?}}}",
                        player_id, action
                    );
                    
                    Command::perform(
                        async move {
                            network.send_action(&match_id, &player_id, action).await
                        },
                        Message::ActionResult,
                    )
                } else {
                    self.add_log("Não conectado!".to_string());
                    Command::none()
                }
            }
            
            Message::ActionResult(result) => {
                match result {
                    Ok(new_state) => {
                        info!("✅ Ação aceita pelo servidor");
                        self.game_state = Some(new_state);
                        self.selected_coord = None;
                        self.input_mode = InputMode::SelectUnit;
                        self.add_log("Ação executada com sucesso!".to_string());
                    }
                    Err(e) => {
                        error!("❌ Erro na ação: {}", e);
                        self.add_log(format!("Erro: {}", e));
                        self.input_mode = InputMode::SelectUnit;
                        self.selected_coord = None;
                    }
                }
                Command::none()
            }
            
            Message::RefreshState => {
                if let Some(match_id) = &self.match_id {
                    let network = self.network.clone();
                    let match_id = match_id.clone();
                    
                    Command::perform(
                        async move {
                            network.get_state(&match_id).await
                        },
                        |result| match result {
                            Ok(state) => Message::ActionResult(Ok(state)),
                            Err(e) => Message::ActionResult(Err(e)),
                        },
                    )
                } else {
                    Command::none()
                }
            }
            
            Message::ClearSelection => {
                self.selected_coord = None;
                self.input_mode = InputMode::SelectUnit;
                self.add_log("Seleção limpa".to_string());
                Command::none()
            }
            
            Message::KeyPressed(key) => {
                info!("⌨️ Tecla pressionada: {}", key);
                // Implementar controle por teclado WASD
                Command::none()
            }
            
            Message::RequestAiMove => {
                if let (Some(match_id), Some(state)) = (&self.match_id, &self.game_state) {
                    if state.turn != self.player_id {
                        info!("🤖 Solicitando movimento da IA...");
                        let network = self.network.clone();
                        let match_id = match_id.clone();
                        let ai_player = state.turn.clone();
                        
                        Command::perform(
                            async move {
                                network.get_ai_action(&match_id, &ai_player).await
                            },
                            Message::AiMoveResult,
                        )
                    } else {
                        Command::none()
                    }
                } else {
                    Command::none()
                }
            }
            
            Message::AiMoveResult(result) => {
                match result {
                    Ok(action) => {
                        info!("🤖 IA escolheu: {:?}", action);
                        self.add_log(format!("IA joga: {:?}", action));
                        Command::perform(async {}, move |_| Message::SendAction(action))
                    }
                    Err(e) => {
                        error!("❌ Erro na IA: {}", e);
                        self.add_log(format!("Erro IA: {}", e));
                        Command::none()
                    }
                }
            }
            
            Message::WebSocketMessage(msg) => {
                info!("📨 WebSocket: {}", msg);
                // Processar mensagem do WebSocket
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg) {
                    if data["type"] == "state_update" {
                        if let Ok(state) = serde_json::from_value::<GameState>(data["state"].clone()) {
                            self.game_state = Some(state);
                            self.add_log("Estado atualizado via WebSocket".to_string());
                        }
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let title = text("RPG ASCII Tático")
            .size(30);

        let status = text(format!("Status: {:?}", self.connection_status))
            .size(16);

        // Botões de controle
        let controls = row![
            button("Conectar").on_press(Message::Connect),
            button("Finalizar Turno").on_press(Message::SendAction(Action::EndTurn)),
            button("IA Jogar").on_press(Message::RequestAiMove),
            button("Limpar Seleção").on_press(Message::ClearSelection),
        ]
        .spacing(10);

        // Renderiza tabuleiro
        let board_view = if let Some(state) = &self.game_state {
            BoardRenderer::render(&state.board, self.selected_coord)
        } else {
            container(text("Aguardando conexão..."))
                .width(Length::Fill)
                .height(400)
                .center_x()
                .center_y()
        };

        // Informações do jogo
        let game_info = if let Some(state) = &self.game_state {
            column![
                text(format!("Turno: {}", state.turn)),
                text(format!("Contador: {}", state.turn_count)),
                text(format!("Fase: {:?}", state.phase)),
            ]
        } else {
            column![text("Jogo não iniciado")]
        };

        // Log de mensagens
        let log_view = scrollable(
            Column::with_children(
                self.message_log
                    .iter()
                    .rev()
                    .take(10)
                    .map(|msg| text(msg).size(12).into())
                    .collect(),
            )
        )
        .height(150);

        // Layout principal
        container(
            column![
                title,
                status,
                controls,
                row![
                    board_view,
                    container(game_info).padding(20),
                ]
                .spacing(20),
                container(log_view).padding(10),
            ]
            .spacing(20)
            .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .into()
    }
}

impl RpgAsciiClient {
    fn add_log(&mut self, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        self.message_log.push(format!("[{}] {}", timestamp, message));
        
        // Limita log a 100 mensagens
        if self.message_log.len() > 100 {
            self.message_log.remove(0);
        }
    }
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("info,client=debug")
        .init();
}

fn main() -> iced::Result {
    RpgAsciiClient::run(Settings::default())
}
