use tatic_lib::{Board, Cell, Coord};
use iced::{
    widget::{button, container, text},
    Color, Length,
};

use crate::Message;

pub struct BoardRenderer;

impl BoardRenderer {
    /// Renderiza o tabuleiro como grid de botões
    pub fn render(board: &Board, selected: Option<Coord>) -> iced::widget::Container<'static, Message> {
        use iced::widget::{column, row};
        
        let mut grid = column![];
        
        // Header com coordenadas
        let mut header = row![];
        header = header.push(text("  ").size(20));
        for x in 0..8 {
            header = header.push(
                container(text(format!("{}", x)).size(16))
                    .width(40)
                    .center_x(Length::Fill)
            );
        }
        grid = grid.push(header);
        
        // Células do tabuleiro
        for y in 0..8 {
            let mut row_widgets = row![];
            
            // Número da linha
            row_widgets = row_widgets.push(
                container(text(format!("{} ", y)).size(16))
                    .width(20)
            );
            
            // Células
            for x in 0..8 {
                let coord = Coord::new(x, y);
                let cell = board.get(&coord).unwrap();
                
                let symbol = match cell {
                    Cell::Unit(unit) => unit.symbol.to_string(),
                    Cell::Empty => ".".to_string(),
                };
                
                let mut cell_button = button(text(symbol).size(20))
                    .on_press(Message::CellClicked(coord))
                    .width(40)
                    .height(40);
                
                // Destaca célula selecionada com estilo customizado
                if Some(coord) == selected {
                    cell_button = cell_button.style(move |_theme, _status| {
                        iced::widget::button::Style {
                            background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.6, 1.0))),
                            text_color: Color::WHITE,
                            border: iced::Border {
                                color: Color::from_rgb(0.0, 0.4, 0.8),
                                width: 2.0,
                                radius: 4.0.into(),
                            },
                            ..Default::default()
                        }
                    });
                }
                
                row_widgets = row_widgets.push(cell_button);
            }
            
            grid = grid.push(row_widgets);
        }
        
        container(grid).padding(10)
    }
    
    /// Renderiza o tabuleiro em ASCII puro (para terminal)
    pub fn render_ascii(board: &Board) -> String {
        let lines = board.to_ascii();
        lines.join("\n")
    }
}
