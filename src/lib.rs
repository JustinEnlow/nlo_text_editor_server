use serde::{Serialize, Deserialize};

pub mod editor;
pub mod document;


pub const MESSAGE_SIZE: usize = 4096;



#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum ServerAction{
    CloseConnection,
    OpenFile(String), // (String, View)? i think we need to supply document rect size, so we can return the correct text to display
    RequestClientViewText,
    UpdateClientView(u16, u16)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerResponse{
    ConnectionSucceeded,
    Acknowledge,
    DisplayView(String),
    Failed(String),
}

#[derive(Debug)]
pub struct View{
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

#[derive(Debug, Default, Clone, Copy)]
pub struct Position{
    x: usize,
    y: usize,
}
impl Position{
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