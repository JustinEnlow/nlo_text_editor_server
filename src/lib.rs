use serde::{Serialize, Deserialize};

pub mod editor;
pub mod document;



#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum ServerAction{
    Backspace,
    CloseConnection,
    CloseDocument,
    CloseDocumentIgnoringChanges,
    CollapseSelectionCursor,
    CommandModeAccept,
    CommandModeBackspace,
    CommandModeDelete,
    CommandModeExit,
    CommandModeInsertChar(char),
    CommandModeMoveCursorLeft,
    CommandModeMoveCursorLineEnd,
    CommandModeMoveCursorLineStart,
    CommandModeMoveCursorRight,
    DecrementFocusedDocument,
    Delete,
    DisplayLineNumbers,
    DisplayStatusBar,
    ExtendSelectionDown,
    ExtendSelectionLeft,
    ExtendSelectionLineEnd,
    ExtendSelectionLineStart,
    ExtendSelectionRight,
    ExtendSelectionUp,
    FindReplaceModeAccept,
    FindReplaceModeBackspace,
    FindReplaceModeDelete,
    FindReplaceModeExit,
    FindReplaceModeInsertChar(char),
    FindReplaceModeMoveCursorLeft,
    FindReplaceModeMoveCursorLineEnd,
    FindReplaceModeMoveCursorLineStart,
    FindReplaceModeMoveCursorRight,
    FindReplaceModeNextInstance,
    FindReplaceModePreviousInstance,
    FindReplaceModeSwitchUtilBarFocus,
    GotoModeAccept,
    GotoModeBackspace,
    GotoModeDelete,
    GotoModeExit,
    GotoModeInsertChar(char),
    GotoModeMoveCursorLeft,
    GotoModeMoveCursorLineEnd,
    GotoModeMoveCursorLineStart,
    GotoModeMoveCursorRight,
    IncrementFocusedDocument,
    InsertChar(char),
    InsertNewline,
    InsertTab,
    MoveCursorDocumentEnd,
    MoveCursorDocumentStart,
    MoveCursorDown,
    MoveCursorLeft,
    MoveCursorLineEnd,
    MoveCursorLineStart,
    MoveCursorPageDown,
    MoveCursorPageUp,
    MoveCursorRight,
    MoveCursorUp,
    MoveCursorWordEnd,
    MoveCursorWordStart,
    NewDocument,
    NoOp,
    OpenFile(String), // (String, View)? i think we need to supply document rect size, so we can return the correct text to display
    OpenNewTerminalWindow,
    Quit,
    QuitIgnoringChanges,
    Save,
    SaveAsModeAccept,
    SaveAsModeBackspace,
    SaveAsModeClear,
    SaveAsModeDelete,
    SaveAsModeInsertChar(char),
    SaveAsModeMoveCursorLeft,
    SaveAsModeMoveCursorLineEnd,
    SaveAsModeMoveCursorLineStart,
    SaveAsModeMoveCursorRight,
    SetModeCommand,
    SetModeFindReplace,
    SetModeGoto,
    WarningModeExit,
    //UpdateViewSize(usize, usize)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerResponse{
    ConnectionSucceeded,
    DisplayView(String),
}

pub struct View{
    horizontal_start: usize,
    vertical_start: usize,
    width: usize,
    height: usize,
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