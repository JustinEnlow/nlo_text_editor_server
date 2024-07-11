use std::path::PathBuf;

use serde::{Serialize, Deserialize};

pub mod editor;
pub mod document;


pub const MESSAGE_SIZE: usize = 8192;//4096;



#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum ServerAction{
    Backspace,
    CloseConnection,
    Delete,
    GoTo(usize),
    OpenFile(/*String*/PathBuf), // (String, View)? i think we need to supply document rect size, so we can return the correct text to display
    //RequestClientViewText,
    UpdateClientViewSize(u16, u16),
    ScrollClientViewDown(usize),
    ScrollClientViewLeft(usize),
    ScrollClientViewRight(usize),
    ScrollClientViewUp(usize),
    //RequestClientCursorPosition,
    MoveCursorDocumentEnd,
    MoveCursorDocumentStart,
    MoveCursorDown,
    MoveCursorUp,
    MoveCursorRight,
    MoveCursorLeft,
    MoveCursorLineEnd,
    MoveCursorLineStart,
    MoveCursorPageDown,
    MoveCursorPageUp,
    InserChar(char),
    InsertNewline,
    InsertTab,
    Save,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerResponse{
    FileOpened(Option<String>, usize), //(filename, document_length)
    ConnectionSucceeded,
    Acknowledge,
    DisplayView(String, String, Option<Position>, Position, bool), //(content, line_numbers, client_cursor_position, document_cursor_position, modified)
    Failed(String), //(reason for failure)
    CursorPosition(Option<Position>, Position), //(client_cursor_position, document_cursor_position)
}

#[derive(Debug)]
pub struct View{
    //origin: Position, //instead of horizontal_start and vertical_start?
    horizontal_start: usize,
    vertical_start: usize,
    width: usize,
    height: usize,
}
impl View{
    pub fn default() -> Self{
        Self {
            horizontal_start: 0,
            vertical_start: 0,
            width: 0,
            height: 0
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct Position{
    x: usize,
    y: usize,
}
impl Position{
    pub fn new(x: usize, y: usize) -> Self{
        Self{x, y}
    }
    pub fn x(&self) -> usize{
        self.x
    }
    pub fn set_x(&mut self, val: usize){
        self.x = val;
    }
    pub fn y(&self) -> usize{
        self.y
    }
    pub fn set_y(&mut self, val: usize){
        self.y = val;
    }
}
impl PartialEq for Position{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl Eq for Position{}