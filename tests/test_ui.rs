#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_input_state_initialization() {
        let input = crate::input::InputState::new();
        assert_eq!(input.cursor.x, 0);
        assert_eq!(input.cursor.y, 0);
        assert!(!input.shift_pressed);
        assert!(!input.ctrl_pressed);
    }
    
    #[test]
    fn test_board_renderer_ascii() {
        let board = tatic_lib::Board::with_initial_setup();
        let ascii = crate::renderer::BoardRenderer::render_ascii(&board);
        
        assert!(ascii.contains("X")); // Player 1 units
        assert!(ascii.contains("O")); // Player 2 units
        assert!(ascii.contains("0 1 2 3 4 5 6 7")); // Header
    }
    
    #[test]
    fn test_network_client_creation() {
        let client = crate::net::NetworkClient::new("http://localhost:3000".to_string());
        // Client criado com sucesso
        assert!(true);
    }
}
