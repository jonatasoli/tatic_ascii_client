use iced::{
    widget::{button, column, container, row, text, Button, Column, Container, Row, Text},
    Alignment, Element, Length,
};

/// Cria painel de informações do jogador
pub fn player_info_panel<'a>(
    player_name: &str,
    units_count: usize,
    is_current_turn: bool,
) -> Container<'a, crate::Message> {
    let title = text(player_name).size(18);
    
    let units = text(format!("Unidades: {}", units_count)).size(14);
    
    let turn_indicator = if is_current_turn {
        text("🎯 Seu turno!").size(16)
    } else {
        text("⏳ Aguardando...").size(14)
    };
    
    container(
        column![title, units, turn_indicator]
            .spacing(5)
            .align_items(Alignment::Start),
    )
    .padding(10)
    .style(iced::theme::Container::Box)
}

/// Cria painel de controles de jogo
pub fn game_controls<'a>() -> Row<'a, crate::Message> {
    row![
        button("⬆ W").width(50),
        button("⬇ S").width(50),
        button("⬅ A").width(50),
        button("➡ D").width(50),
        button("✓ Enter").width(80),
        button("✗ ESC").width(80),
    ]
    .spacing(5)
}

/// Cria indicador de status da conexão
pub fn connection_indicator<'a>(is_connected: bool) -> Container<'a, crate::Message> {
    let (icon, text_str, color) = if is_connected {
        ("🟢", "Conectado", [0.0, 0.8, 0.0, 1.0])
    } else {
        ("🔴", "Desconectado", [0.8, 0.0, 0.0, 1.0])
    };
    
    container(
        row![
            text(icon).size(12),
            text(text_str).size(12)
        ]
        .spacing(5)
        .align_items(Alignment::Center),
    )
    .padding(5)
}
