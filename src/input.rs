use tatic_lib::Coord;
use iced::keyboard::{self, Key};
use iced::Event;

/// Estado do input do teclado
#[derive(Debug, Clone)]
pub struct InputState {
    pub cursor: Coord,
    pub shift_pressed: bool,
    pub ctrl_pressed: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            cursor: Coord::new(0, 0),
            shift_pressed: false,
            ctrl_pressed: false,
        }
    }
    
    /// Processa evento de teclado
    pub fn handle_keyboard_event(&mut self, event: Event) -> Option<crate::Message> {
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                match key {
                    Key::Character(c) => match c.as_str() {
                        "w" | "W" => {
                            if self.cursor.y > 0 {
                                self.cursor.y -= 1;
                            }
                            Some(crate::Message::KeyPressed('w'))
                        }
                        "s" | "S" => {
                            if self.cursor.y < 7 {
                                self.cursor.y += 1;
                            }
                            Some(crate::Message::KeyPressed('s'))
                        }
                        "a" | "A" => {
                            if self.cursor.x > 0 {
                                self.cursor.x -= 1;
                            }
                            Some(crate::Message::KeyPressed('a'))
                        }
                        "d" | "D" => {
                            if self.cursor.x < 7 {
                                self.cursor.x += 1;
                            }
                            Some(crate::Message::KeyPressed('d'))
                        }
                        _ => None,
                    }
                    Key::Named(keyboard::key::Named::Enter) => {
                        Some(crate::Message::CellClicked(self.cursor))
                    }
                    Key::Named(keyboard::key::Named::Escape) => {
                        Some(crate::Message::ClearSelection)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
