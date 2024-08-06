use std::path::PathBuf;

use serde::{Serialize, Deserialize};

pub mod editor;
pub mod document;
mod selection;
mod movement;


pub const MESSAGE_SIZE: usize = 8192;//4096;



pub enum Operation{
    Move(usize),
    Delete(usize),
    Insert(String),
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum ServerAction{
    Backspace,
    CloseConnection,
    Delete,
    GoTo{line_number: usize},
    OpenFile{file_path: PathBuf},
    UpdateClientViewSize{width: u16, height: u16},
    ScrollClientViewDown{amount: usize},
    ScrollClientViewLeft{amount: usize},
    ScrollClientViewRight{amount: usize},
    ScrollClientViewUp{amount: usize},
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
    FileOpened{file_name: Option<String>, document_length: usize},
    ConnectionSucceeded,
    Acknowledge,
    DisplayView{content: String, line_numbers: String, client_cursor_positions: Vec<Position>, document_cursor_position: Position, modified: bool},
    Failed(String), //(reason for failure)
    CursorPosition{client_cursor_positions: Vec<Position>, document_cursor_position: Position}
}

#[derive(Debug, Default, Clone)]
pub struct View{
    horizontal_start: usize,
    vertical_start: usize,
    width: usize,
    height: usize,
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