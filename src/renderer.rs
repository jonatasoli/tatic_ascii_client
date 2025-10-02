use tatic_lib::{Board, Cell, Coord};
use iced::{
    widget::{button, column, container, row, text, Button, Column, Container, Row, Text},
    Alignment, Element, Length,
};

use crate::Message;

pub struct BoardRenderer;

impl BoardRenderer {
    /// Renderiza o tabuleiro como grid de botões
    pub fn render(board: &Board, selected: Option<Coord>) -> Container<'static, Message> {
        let mut grid = Column::new();
        
        // Header com coordenadas
        let mut header = Row::new();
        header = header.push(text("  ").size(20));
        for x in 0..8 {
            header = header.push(
                container(text(format!("{}", x)).size(16))
                    .width(40)
                    .center_x()
            );
        }
        grid = grid.push(header);
        
        // Células do tabuleiro
        for y in 0..8 {
            let mut row_widgets = Row::new();
            
            // Número da linha
            row_widgets = row_widgets.push(
                container(text(format!("{} ", y)).size(16))
                    .width(20)
            );
            
            // Células
            for x in 0..8 {
                let coord = Coord::new(x, y);
                let cell = board.get(&coord).unwrap();
                
                let (symbol, color) = match cell {
                    Cell::Unit(unit) => {
                        let color = if unit.owner == "player1" {
                            [0.2, 0.2, 0.8, 1.0] // Azul
                        } else {
                            [0.8, 0.2, 0.2, 1.0] // Vermelho
                        };
                        (unit.symbol.to_string(), color)
                    }
                    Cell::Empty => (".".to_string(), [0.3, 0.3, 0.3, 1.0]),
                };
                
                let mut cell_button = button(text(symbol).size(20))
                    .on_press(Message::CellClicked(coord))
                    .width(40)
                    .height(40);
                
                // Destaca célula selecionada
                if Some(coord) == selected {
                    cell_button = cell_button.style(iced::theme::Button::Primary);
                }
                
                row_widgets = row_widgets.push(cell_button);
            }
            
            grid = grid.push(row_widgets);
        }
        
        container(grid)
            .padding(10)
            .style(iced::theme::Container::Box)
    }
    
    /// Renderiza o tabuleiro em ASCII puro (para terminal)
    pub fn render_ascii(board: &Board) -> String {
        let lines = board.to_ascii();
        lines.join("\n")
    }
}
