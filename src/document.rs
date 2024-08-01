use crate::{Position, View};
use std::fs::{self, File};
use std::{error::Error, fmt::Display};
use std::io::{BufReader, Write};
use std::path::PathBuf;
use unicode_segmentation::UnicodeSegmentation;
use ropey::{Rope, RopeSlice};

// tab keypress inserts the number of spaces specified in TAB_WIDTH into the focused document
pub const TAB_WIDTH: usize = 4;



#[derive(PartialEq, Debug)]
struct DocumentCursor{
    head: Position,
    anchor: Position,
}
impl DocumentCursor{
    pub fn new(head: Position, anchor: Position) -> Self{
        Self{
            head,
            anchor
        }
    }
}



#[derive(Default, PartialEq, Clone, Debug)]
struct RopeCursor{
    anchor: usize,
    head: usize,
    stored_line_position: usize,
}



pub struct Document{
    text: Rope,
    file_name: Option<String>,
    modified: bool,
    rope_cursors: Vec<RopeCursor>,
    client_view: View,
}
impl Default for Document{
    fn default() -> Self {
        Self{
            text: Rope::new(),
            file_name: None,
            modified: false,
            rope_cursors: vec![RopeCursor::default()],
            client_view: View::default(),
        }
    }
}
impl Document{
    pub fn open(path: &PathBuf) -> Result<Self, Box<dyn Error>>{
        let text = Rope::from_reader(BufReader::new(File::open(path)?))?;
    
        Ok(Self{
            text,
            file_name: Some(path.to_string_lossy().to_string()),
            modified: false,
            rope_cursors: vec![RopeCursor::default()],
            client_view: View::default(),
        })
    }

    pub fn text(&self) -> RopeSlice{
        self.text.slice(..)
    }

    pub fn file_name(&self) -> Option<String>{
        self.file_name.clone()
    }

    /// Translates a 1 dimensional rope cursor to a 2 dimensional document cursor
    fn rope_cursor_position_to_document_cursor_position(rope_cursor: RopeCursor, text: RopeSlice) -> DocumentCursor{
        let line_number_head = text.char_to_line(rope_cursor.head);
        let line_number_anchor = text.char_to_line(rope_cursor.anchor);

        let line_at_head_start_idx = text.line_to_char(line_number_head);
        let line_at_anchor_start_idx = text.line_to_char(line_number_anchor);

        DocumentCursor{
            head: Position::new(
                rope_cursor.head - line_at_head_start_idx, 
                line_number_head
            ),
            anchor: Position::new(
                rope_cursor.anchor - line_at_anchor_start_idx,
                line_number_anchor
            )
        }
    }
    //TODO: return head and anchor positions
    //TODO: return Vec<Position> document cursor positions
    pub fn document_cursor_position(&self) -> Position{
        let cursor = self.rope_cursors.last().unwrap();
        let document_cursor = Document::rope_cursor_position_to_document_cursor_position(cursor.clone(), self.text.slice(..));
        
        Position::new(
            document_cursor.head.x.saturating_add(1), 
            document_cursor.head.y.saturating_add(1)
        )
    }
    fn clear_cursors_except_main(cursors: &mut Vec<RopeCursor>){
        for x in (0..cursors.len()).rev(){
            if x != 0{
                cursors.pop();
            }
        }
    }
    fn line_width_excluding_newline(line: RopeSlice) -> usize{
        let mut line_width = 0;
        for char in line.chars(){
            if char != '\n'{
                line_width = line_width + 1;
            }
        }
        line_width
    }
    /// Sets rope cursor to a 0 based line number
    fn set_rope_cursor_position_from_line_number(mut rope_cursor: RopeCursor, line_number: usize, text: RopeSlice) -> RopeCursor{
        if line_number < text.len_lines(){ //is len lines 1 based?
            let start_of_line = text.line_to_char(line_number);
            let line = text.line(line_number);
            //let mut line_width = 0;
            //for char in line.chars(){
            //    if char != '\n'{
            //        line_width = line_width + 1;
            //    }
            //}
            let line_width = Document::line_width_excluding_newline(line);
            if rope_cursor.stored_line_position < line_width{
                rope_cursor.anchor = start_of_line + rope_cursor.stored_line_position;
                rope_cursor.head = start_of_line + rope_cursor.stored_line_position;
            }else{
                rope_cursor.anchor = start_of_line + line_width;
                rope_cursor.head = start_of_line + line_width;
            }
        }
    
        rope_cursor
    }

    //pub fn add_cursor_on_line_above(&mut self){
    //    self.cursors.push(
    //        Cursor::new(
    //            Position::new(self.cursors.last().unwrap().head.x, self.cursors.last().unwrap().head.y.saturating_sub(1)),
    //            Position::new(self.cursors.last().unwrap().head.x, self.cursors.last().unwrap().head.y.saturating_sub(1))
    //        )
    //    );
    //
    //    //unwrapping because this is guaranteed to have a cursor because one was just added
    //    let cursor = self.cursors.last_mut().unwrap();
    //    *cursor = Document::clamp_cursor_to_line_end(cursor, &self.lines);
    //}
    //pub fn add_cursor_on_line_below(&mut self){
    //
    //}

    //currently using insert_char('\n') for enter. not currently handling auto indent
            //pub fn enter(&mut self){        
            //    for cursor in self.cursors.iter_mut(){
            //        Document::enter_at_cursor(cursor, &mut self.lines, &mut self.modified);
            //    }
            //}
            // auto indent doesn't work correctly if previous line has only whitespace characters
            // also doesn't auto indent for first line of function bodies, because function declaration
            // is at lower indentation level
            //fn enter_at_cursor(cursor: &mut Cursor, lines: &mut Vec<String>, modified: &mut bool){
            //    *modified = true;
            //    
            //    match lines.get_mut(cursor.head.y){
            //        Some(line) => {
            //            let start_of_line = get_first_non_whitespace_character_index(line);
            //            let mut modified_current_line: String = String::new();
            //            let mut new_line: String = String::new();
            //            for (index, grapheme) in line[..].graphemes(true).enumerate(){
            //                if index < cursor.head.x{
            //                    modified_current_line.push_str(grapheme);
            //                }
            //                else{
            //                    new_line.push_str(grapheme);
            //                }
            //            }
            //            *line = modified_current_line;
            //            lines.insert(cursor.head.y.saturating_add(1), new_line);
            //            Document::move_cursor_right(cursor, &lines);
            //            // auto indent
            //            if start_of_line != 0{
            //                for _ in 0..start_of_line{
            //                    Document::insert_char_at_cursor(' ', cursor, lines, modified);
            //                }
            //            }
            //        }
            //        None => panic!("No line at cursor position. This should be impossible")
            //    }
            //}

    pub fn insert_char(&mut self, c: char){
        self.modified = true;
        
        for cursor in self.rope_cursors.iter_mut(){
            (*cursor, self.text) = Document::insert_char_at_cursor(cursor.clone(), self.text.slice(..), c);
        }
    }
    fn insert_char_at_cursor(mut rope_cursor: RopeCursor, text: RopeSlice, char: char) -> (RopeCursor, Rope){
        let mut new_text = Rope::from(text);
        new_text.insert_char(rope_cursor.head, char);
        rope_cursor = Document::move_cursor_right(rope_cursor, new_text.slice(..));

        (rope_cursor, new_text)
    }

    pub fn tab(&mut self){
        self.modified = true;

        for cursor in self.rope_cursors.iter_mut(){
            let tab_distance = distance_to_next_multiple_of_tab_width(cursor.clone());
            let modified_tab_width = if tab_distance > 0 && tab_distance < TAB_WIDTH{
                tab_distance
            }else{
                TAB_WIDTH
            };
            for _ in 0..modified_tab_width{
                (*cursor, self.text) = Document::insert_char_at_cursor(cursor.clone(), self.text.slice(..), ' ');
            }
        }
    }

    //TODO: don't set modified true, if no deletion actually performed
    pub fn delete(&mut self){
        self.modified = true;

        for cursor in self.rope_cursors.iter_mut(){
            self.text = Document::delete_at_cursor(cursor.clone(), self.text.slice(..));
        }
    }
    //TODO: handle selection deletion, not just char deletion
    //TODO: ensure we cannot delete at EOF
    fn delete_at_cursor(cursor: RopeCursor, text: RopeSlice) -> Rope{
        let mut new_text = Rope::from(text);

        if cursor.head < text.len_chars(){
            new_text.remove(cursor.head..cursor.head+1);
        }

        new_text
    }

    pub fn backspace(&mut self){
        self.modified = true;

        for cursor in self.rope_cursors.iter_mut(){
            let cursor_line_position = cursor.head - self.text.line_to_char(self.text.char_to_line(cursor.head));
            
            if cursor_line_position >= TAB_WIDTH
            // handles case where user adds a space after a tab, and wants to delete only the space
            && cursor_line_position % TAB_WIDTH == 0
            // if previous 4 chars are spaces, delete 4. otherwise, use default behavior
            && slice_is_all_spaces(
                self.text.line(self.text.char_to_line(cursor.head)).as_str().unwrap(),
                cursor_line_position - TAB_WIDTH,
                cursor_line_position
            ){
                for _ in 0..TAB_WIDTH{
                    *cursor = Document::move_cursor_left(cursor.clone(), self.text.slice(..));
                    self.text = Document::delete_at_cursor(cursor.clone(), self.text.slice(..));
                }
            }
            else if cursor.head > 0{
                *cursor = Document::move_cursor_left(cursor.clone(), self.text.slice(..));
                self.text = Document::delete_at_cursor(cursor.clone(), self.text.slice(..));
            }
        }
    }

    pub fn move_cursors_up(&mut self){
        for cursor in self.rope_cursors.iter_mut(){
            *cursor = Document::move_cursor_up(cursor.clone(), self.text.slice(..));
        }
    }
    fn move_cursor_up(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        let line_number = text.char_to_line(rope_cursor.head);
        let previous_line_number = line_number.saturating_sub(1);
        rope_cursor = Document::set_rope_cursor_position_from_line_number(rope_cursor, previous_line_number, text.slice(..));

        rope_cursor
    }

    pub fn move_cursors_down(&mut self){
        for cursor in self.rope_cursors.iter_mut(){
            *cursor = Document::move_cursor_down(cursor.clone(), self.text.slice(..));
        }
    }
    fn move_cursor_down(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        let line_number = text.char_to_line(rope_cursor.head);
        let next_line_number = line_number.saturating_add(1);
        rope_cursor = Document::set_rope_cursor_position_from_line_number(rope_cursor, next_line_number, text);

        rope_cursor
    }

    pub fn move_cursors_right(&mut self){
        for cursor in self.rope_cursors.iter_mut(){
            *cursor = Document::move_cursor_right(cursor.clone(), self.text.slice(..));
        }
    }
    fn move_cursor_right(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        if /*(*/rope_cursor.head.saturating_add(1) < text.len_chars()
        || rope_cursor.head.saturating_add(1) == text.len_chars()/*)*/
        //&& (rope_cursor.anchor.saturating_add(1) < text.len_chars()
        //|| rope_cursor.anchor.saturating_add(1) == text.len_chars())
        {
            rope_cursor.head = rope_cursor.head.saturating_add(1);
            rope_cursor.anchor = rope_cursor.anchor.saturating_add(1);
            let line_start = text.line_to_char(text.char_to_line(rope_cursor.head));
            rope_cursor.stored_line_position = rope_cursor.head.saturating_sub(line_start);
        }

        rope_cursor
    }

    pub fn move_cursors_left(&mut self){
        for cursor in self.rope_cursors.iter_mut(){
            *cursor = Document::move_cursor_left(cursor.clone(), self.text.slice(..));
        }
    }
    fn move_cursor_left(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        rope_cursor.head = rope_cursor.head.saturating_sub(1);
        rope_cursor.anchor = rope_cursor.anchor.saturating_sub(1);
        let line_start = text.line_to_char(text.char_to_line(rope_cursor.head));
        rope_cursor.stored_line_position = rope_cursor.head.saturating_sub(line_start);

        rope_cursor
    }

    pub fn move_cursors_page_up(&mut self){
        for cursor in self.rope_cursors.iter_mut(){
            *cursor = Document::move_cursor_page_up(cursor.clone(), self.text.slice(..), self.client_view.clone())
        }
    }
    fn move_cursor_page_up(mut rope_cursor: RopeCursor, text: RopeSlice, client_view: View) -> RopeCursor{
        let line_number = text.char_to_line(rope_cursor.head);
        let goal_line_number = line_number.saturating_sub(client_view.height.saturating_sub(1));
        rope_cursor = Document::set_rope_cursor_position_from_line_number(rope_cursor, goal_line_number, text);

        rope_cursor
    }

    pub fn move_cursors_page_down(&mut self){
        for cursor in self.rope_cursors.iter_mut(){
            *cursor = Document::move_cursor_page_down(cursor.clone(), self.text.slice(..), self.client_view.clone());
        }
    }
    fn move_cursor_page_down(mut rope_cursor: RopeCursor, text: RopeSlice, client_view: View) -> RopeCursor{
        let document_length = text.len_lines();
        let line_number = text.char_to_line(rope_cursor.head);
        let goal_line_number = if line_number.saturating_add(client_view.height) <= document_length{
            line_number.saturating_add(client_view.height.saturating_sub(1))
        }else{
            document_length.saturating_sub(1)
        };
        rope_cursor = Document::set_rope_cursor_position_from_line_number(rope_cursor, goal_line_number, text);

        rope_cursor
    }

    pub fn move_cursors_home(&mut self){
        for cursor in self.rope_cursors.iter_mut(){
            *cursor = Document::move_cursor_home(cursor.clone(), self.text.slice(..));
        }
    }
    fn move_cursor_home(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        let line_number = text.char_to_line(rope_cursor.head);
        let line_start = text.line_to_char(line_number);
        let text_start_offset = get_first_non_whitespace_character_index(text.line(line_number));
        let text_start = line_start.saturating_add(text_start_offset);

        if rope_cursor.head == text_start{
            //TODO: break out into own move_cursor_line_start fn?
            rope_cursor.head = line_start;
            rope_cursor.anchor = line_start;
        }else{
            //TODO: break out into own move_cursor_line_text_start fn?
            rope_cursor.head = text_start;
            rope_cursor.anchor = text_start;
        }
        rope_cursor.stored_line_position = rope_cursor.head.saturating_sub(line_start);

        rope_cursor
    }

    pub fn move_cursors_end(&mut self){
        for cursor in self.rope_cursors.iter_mut(){
            *cursor = Document::move_cursor_end(cursor.clone(), self.text.slice(..));
        }
    }
    fn move_cursor_end(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        let line_number = text.char_to_line(rope_cursor.head);
        let line = text.line(line_number);
        //let mut line_width = 0;
        //for char in line.chars(){
        //    if char != '\n'{
        //        line_width = line_width + 1;
        //    }
        //}
        let line_width = Document::line_width_excluding_newline(line);
        let line_start = text.line_to_char(line_number);
        let line_end = line_start.saturating_add(line_width);

        rope_cursor.head = line_end;
        rope_cursor.anchor = line_end;
        rope_cursor.stored_line_position = line_end.saturating_sub(line_start);

        rope_cursor
    }

    pub fn move_cursors_document_start(&mut self){
        Document::clear_cursors_except_main(&mut self.rope_cursors);
        match self.rope_cursors.get_mut(0){
            Some(cursor) => {
                *cursor = Document::move_cursor_document_start(cursor.clone());
            }
            None => panic!("No cursor at 0 index. This should be impossible.")
        }
    }
    fn move_cursor_document_start(mut rope_cursor: RopeCursor) -> RopeCursor{
        rope_cursor.head = 0;
        rope_cursor.anchor = 0;
        rope_cursor.stored_line_position = 0;

        rope_cursor
    }

    pub fn move_cursors_document_end(&mut self){
        Document::clear_cursors_except_main(&mut self.rope_cursors);
        match self.rope_cursors.get_mut(0){
            Some(cursor) => {
                *cursor = Document::move_cursor_document_end(cursor.clone(), self.text.slice(..));
            }
            None => panic!("No cursor at 0 index. This should be impossible.")
        }
    }
    fn move_cursor_document_end(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        rope_cursor.head = text.len_chars();
        rope_cursor.anchor = text.len_chars();
        let line_start = text.line_to_char(text.char_to_line(rope_cursor.head));
        rope_cursor.stored_line_position = text.len_chars().saturating_sub(line_start);

        rope_cursor
    }

    //pub fn extend_selections_right(&mut self){
    //    for cursor in self.cursors.iter_mut(){
    //        Document::extend_selection_right_at_cursor(cursor, &self.lines);
    //    }
    //}
    fn extend_selection_right(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        if /*(*/rope_cursor.head.saturating_add(1) < text.len_chars()
        || rope_cursor.head.saturating_add(1) == text.len_chars()/*)*/
        //&& (rope_cursor.anchor.saturating_add(1) < text.len_chars()
        //|| rope_cursor.anchor.saturating_add(1) == text.len_chars())
        {
            rope_cursor.head = rope_cursor.head.saturating_add(1);
            let line_start = text.line_to_char(text.char_to_line(rope_cursor.head));
            rope_cursor.stored_line_position = rope_cursor.head.saturating_sub(line_start);
        }

        rope_cursor
    }

    //pub fn extend_selections_left(&mut self){
    //    for cursor in self.cursors.iter_mut(){
    //        Document::extend_selection_left_at_cursor(cursor, &self.lines);
    //    }
    //}
    fn extend_selection_left(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        rope_cursor.head = rope_cursor.head.saturating_sub(1);
        let line_start = text.line_to_char(text.char_to_line(rope_cursor.head));
        rope_cursor.stored_line_position = rope_cursor.head.saturating_sub(line_start);

        rope_cursor
    }

    //pub fn extend_selections_up(&mut self){
    //    for cursor in self.cursors.iter_mut(){
    //        Document::extend_selection_up_at_cursor(cursor, &self.lines);
    //    }
    //}
    fn extend_selection_up(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        let line_number = text.char_to_line(rope_cursor.head);
        let previous_line_number = line_number.saturating_sub(1);
        if previous_line_number < text.len_lines(){
            let start_of_line = text.line_to_char(previous_line_number);
            let line = text.line(previous_line_number);
            //let mut line_width = 0;
            //for char in line.chars(){
            //    if char != '\n'{
            //        line_width = line_width + 1;
            //    }
            //}
            let line_width = Document::line_width_excluding_newline(line);
            if rope_cursor.stored_line_position < line_width{
                rope_cursor.head = start_of_line + rope_cursor.stored_line_position;
            }else{
                rope_cursor.head = start_of_line + line_width;
            }
        }

        rope_cursor
    }

    //pub fn extend_selections_down(&mut self){
    //    for cursor in self.cursors.iter_mut(){
    //        Document::extend_selection_down_at_cursor(cursor, &self.lines);
    //    }
    //}
    fn extend_selection_down(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        let line_number = text.char_to_line(rope_cursor.head);
        let next_line_number = line_number.saturating_add(1);
        if next_line_number < text.len_lines(){
            let start_of_line = text.line_to_char(next_line_number);
            let line = text.line(next_line_number);
            //let mut line_width = 0;
            //for char in line.chars(){
            //    if char != '\n'{
            //        line_width = line_width + 1;
            //    }
            //}
            let line_width = Document::line_width_excluding_newline(line);
            if rope_cursor.stored_line_position < line_width{
                rope_cursor.head = start_of_line + rope_cursor.stored_line_position;
            }else{
                rope_cursor.head = start_of_line + line_width;
            }
        }

        rope_cursor
    }

    //pub fn extend_selections_home(&mut self){
    //    for cursor in self.cursors.iter_mut(){
    //        Document::extend_selection_home_at_cursor(cursor, &self.lines);
    //    }
    //}
    fn extend_selection_home(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        let line_number = text.char_to_line(rope_cursor.head);
        let line_start = text.line_to_char(line_number);
        let text_start_offset = get_first_non_whitespace_character_index(text.line(line_number));
        let text_start = line_start.saturating_add(text_start_offset);

        if rope_cursor.head == text_start{
            rope_cursor.head = line_start;
        }else{
            rope_cursor.head = text_start;
        }
        rope_cursor.stored_line_position = rope_cursor.head.saturating_sub(line_start);

        rope_cursor
    }

    //pub fn extend_selections_end(&mut self){
    //    for cursor in self.cursors.iter_mut(){
    //        Document::extend_selection_end_at_cursor(cursor, &self.lines);
    //    }
    //}
    fn extend_selection_end(mut rope_cursor: RopeCursor, text: RopeSlice) -> RopeCursor{
        let line_number = text.char_to_line(rope_cursor.head);
        let line = text.line(line_number);
        //let mut line_width = 0;
        //for char in line.chars(){
        //    if char != '\n'{
        //        line_width = line_width + 1;
        //    }
        //}
        let line_width = Document::line_width_excluding_newline(line);
        let line_start = text.line_to_char(line_number);
        let line_end = line_start.saturating_add(line_width);

        rope_cursor.head = line_end;
        rope_cursor.stored_line_position = line_end.saturating_sub(line_start);

        rope_cursor
    }

    //pub fn _extend_selections_page_up(&mut self){}

    //pub fn _extend_selections_page_down(&mut self){}

    pub fn collapse_selection_cursors(&mut self){
        for cursor in self.rope_cursors.iter_mut(){
            *cursor = Document::collapse_selection_cursor(cursor.clone());
        }
    }
    //fn collapse_selection_cursor(cursor: &mut Cursor){
    //    if cursor.head.y == cursor.anchor.y{
    //        cursor.anchor.x = cursor.stored_line_position;
    //        cursor.head.x = cursor.stored_line_position;
    //    }else{
    //        cursor.anchor.x = cursor.stored_line_position;
    //        cursor.head.x = cursor.stored_line_position;
    //        cursor.anchor.y = cursor.head.y;
    //    }
    //}
    fn collapse_selection_cursor(mut cursor: RopeCursor) -> RopeCursor{
        cursor.anchor = cursor.head;

        cursor
    }

    //pub fn save(&mut self) -> Result<(), Box<dyn Error>>{
    //    if let Some(file_name) = &self.file_name{ // does nothing if file_name is None
    //        let mut file = fs::File::create(file_name)?;
    //        
    //        for line in &self.lines {
    //            file.write_all(line.as_bytes())?;
    //            file.write_all(b"\n")?;
    //        }
    //        
    //        self.modified = false;
    //    }
    //    
    //    Ok(())
    //}

    pub fn go_to(&mut self, line_number: usize){
        Document::clear_cursors_except_main(&mut self.rope_cursors);
        match self.rope_cursors.get_mut(0){
            Some(cursor) => {
                *cursor = Document::set_rope_cursor_position_from_line_number(
                    cursor.clone(), 
                    line_number, 
                    self.text.slice(..)
                );
            }
            None => panic!("No cursor at 0 index. This should be impossible.")
        }
    }

    pub fn is_modified(&self) -> bool{
        self.modified
    }

    pub fn scroll_client_view_down(&mut self, amount: usize){
        //if self.client_view.vertical_start + self.client_view.height + amount <= self.lines.len(){
        if self.client_view.vertical_start + self.client_view.height + amount <= self.text.len_lines(){
            self.client_view.vertical_start = self.client_view.vertical_start.saturating_add(amount);
        }
    }
    pub fn scroll_client_view_left(&mut self, amount: usize){
        self.client_view.horizontal_start = self.client_view.horizontal_start.saturating_sub(amount);
    }
    pub fn scroll_client_view_right(&mut self, amount: usize){
        let mut longest = 0;
        //for line in &self.lines{
        for line in self.text.lines(){
            //if line.len() > longest{
            //    longest = line.len();
            //}
            //let mut line_width = 0;
            //for char in line.chars(){
            //    if char != '\n'{
            //        line_width = line_width + 1;
            //    }
            //}
            let line_width = Document::line_width_excluding_newline(line);

            if line_width > longest{
                longest = line_width;
            }
        }

        if self.client_view.horizontal_start + self.client_view.width + amount <= longest{
            self.client_view.horizontal_start = self.client_view.horizontal_start.saturating_add(amount);
        }
    }
    pub fn scroll_client_view_up(&mut self, amount: usize){
        self.client_view.vertical_start = self.client_view.vertical_start.saturating_sub(amount);
    }

    pub fn scroll_view_following_cursor(&mut self) -> bool{
        // following last cursor pushed to cursors vec
        //let cursor = self.cursors.last().expect("No cursor. This should be impossible");
        let cursor = Document::rope_cursor_position_to_document_cursor_position(self.rope_cursors.last().expect("No cursor. This should be impossible.").clone(), self.text.slice(..));
        //

        let mut should_update_client_view = false;

        if cursor.head.y() < self.client_view.vertical_start{
            self.client_view.vertical_start = cursor.head.y();
            should_update_client_view = true;
        }
        else if cursor.head.y() >= self.client_view.vertical_start.saturating_add(self.client_view.height){
            self.client_view.vertical_start = cursor.head.y().saturating_sub(self.client_view.height).saturating_add(1);
            should_update_client_view = true;
        }
    
        if cursor.head.x() < self.client_view.horizontal_start{
            self.client_view.horizontal_start = cursor.head.x();
            should_update_client_view = true;
        }
        else if cursor.head.x() >= self.client_view.horizontal_start.saturating_add(self.client_view.width){
            self.client_view.horizontal_start = cursor.head.x().saturating_sub(self.client_view.width).saturating_add(1);
            should_update_client_view = true;
        }

        should_update_client_view
    }

    pub fn set_client_view_size(&mut self, width: usize, height: usize){
        self.client_view.width = width;
        self.client_view.height = height;
    }

    pub fn get_client_view_text(&self) -> String{
        let mut client_view_text = String::new();
        for (y, line) in self.text.lines().enumerate(){
            let mut bounded_line = String::new();
            if y < self.client_view.vertical_start{}
            else if y > (self.client_view.height.saturating_sub(1) + self.client_view.vertical_start){/*can return early, because we're past our view */}
            else{
                for (x, char) in line.chars().enumerate(){
                    if x < self.client_view.horizontal_start{}
                    else if x > (self.client_view.width.saturating_sub(1) + self.client_view.horizontal_start){}
                    else{
                        if char != '\n'{
                            bounded_line.push(char);
                        }
                    }
                }
                client_view_text.push_str(format!("{}\n", bounded_line).as_str());
            }
        }

        client_view_text
    }

    pub fn get_client_view_line_numbers(&self)-> String{
        let mut client_view_line_numbers = String::new();
        for (y, _) in self.text.lines().enumerate(){
            if y < self.client_view.vertical_start{}
            else if y > (self.client_view.height.saturating_sub(1) + self.client_view.vertical_start){/*potential early return*/}
            else{
                client_view_line_numbers.push_str(&format!("{}\n", y.saturating_add(1)))
            }
        }

        client_view_line_numbers
    }

    pub fn get_client_cursor_positions(&self) -> Vec<Position>{
        let mut positions = Vec::new();
        for cursor in &self.rope_cursors{
            if let Some(client_cursor) = Document::client_view_cursor_position(
                Document::rope_cursor_position_to_document_cursor_position(
                    cursor.clone(), 
                    self.text.slice(..)
                ), 
                self.client_view.clone()
            ){
                positions.push(client_cursor);
            }
        }
        positions
    }
    //TODO: return head and anchor so selections can be displayed
    // translates a document cursor position to a client view cursor position. if outside client view, returns None
    fn client_view_cursor_position(doc_cursor: DocumentCursor, client_view: View) -> Option<Position>{
        if doc_cursor.head.x >= client_view.horizontal_start
        && doc_cursor.head.x < client_view.horizontal_start.saturating_add(client_view.width)
        && doc_cursor.head.y >= client_view.vertical_start
        && doc_cursor.head.y < client_view.vertical_start.saturating_add(client_view.height){
            Some(Position{
                x: doc_cursor.head.x.saturating_sub(client_view.horizontal_start),
                y: doc_cursor.head.y.saturating_sub(client_view.vertical_start)
            })
        }else{None}
    }
}

//TODO: handle graphemes instead of chars?
fn get_first_non_whitespace_character_index(line: RopeSlice) -> usize{
    if line.len_chars() == 0{return 0;}

    for (index, char) in line.chars().enumerate(){
        if char == ' '{/*do nothing*/}
        else{
            return index;
        }
    }

    0
}

fn slice_is_all_spaces(line: &str, start_of_slice: usize, end_of_slice: usize) -> bool{
    for grapheme in line[start_of_slice..end_of_slice].graphemes(true){
        if grapheme != " "{
            return false;
        }
    }

    true
}

//TODO: calculate cursor_line_position instead of using stored_line_position
//fn distance_to_next_multiple_of_tab_width(cursor: &Cursor) -> usize{
fn distance_to_next_multiple_of_tab_width(cursor: RopeCursor) -> usize{
    //if cursor.head.x % TAB_WIDTH != 0{
    if cursor.stored_line_position % TAB_WIDTH != 0{
        //TAB_WIDTH - (cursor.head.x % TAB_WIDTH)
        TAB_WIDTH - (cursor.stored_line_position % TAB_WIDTH)
    }else{
        0
    }
}





//#[cfg(test)]
//mod tests{
//    use crate::{document::Document, Position};

//CHECK ROPEY BEHAVIOR
    #[test]
    fn check_ropey_behavior(){
        let text = Rope::from("idk\nsomething\nelse");
        assert!(text.len_lines() == 3);

        let line = text.line(text.char_to_line(5));
        let line_width = line.len_chars();
        println!("line width: {}", line_width);
        assert!(line_width == 10);
        let last_char = line.char(9);
        assert!(last_char == '\n');
    }

//DOCUMENT CURSOR POSITION FROM ROPE CURSOR POSITION
    #[test]
    fn document_cursor_position_works_when_rope_cursor_head_and_anchor_same_and_on_same_line(){
        let text = Rope::from("idk\nsomething");
        let rope_cursor = RopeCursor{anchor: 2, head: 2, stored_line_position: 2};  //id[]k\nsomething
        let doc_cursor = Document::rope_cursor_position_to_document_cursor_position(rope_cursor, text.slice(..));
        let expected_doc_cursor = DocumentCursor::new(Position::new(2, 0), Position::new(2, 0));
        /*
        id[]k
        something
        */
        println!("expected: {expected_doc_cursor:?}\ngot: {doc_cursor:?}");
        assert!(doc_cursor == expected_doc_cursor);
    }
    #[test]
    fn document_cursor_position_works_when_rope_cursor_head_and_anchor_different_but_on_same_line(){
        let text = Rope::from("idk\nsomething");
        let rope_cursor = RopeCursor{anchor: 1, head: 2, stored_line_position: 2};  //i[d]k\nsomething
        let doc_cursor = Document::rope_cursor_position_to_document_cursor_position(rope_cursor, text.slice(..));
        let expected_doc_cursor = DocumentCursor::new(Position::new(2, 0), Position::new(1, 0));
        /*
        i[d]k
        something
        */
        println!("expected: {expected_doc_cursor:?}\ngot: {doc_cursor:?}");
        assert!(doc_cursor == expected_doc_cursor);
    }
    #[test]
    fn document_cursor_position_works_when_rope_cursor_head_and_anchor_same_but_on_new_line(){
        let text = Rope::from("idk\nsomething");
        let rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0};  //idk\n[]something
        let doc_cursor = Document::rope_cursor_position_to_document_cursor_position(rope_cursor, text.slice(..));
        let expected_doc_cursor = DocumentCursor::new(Position::new(0, 1), Position::new(0, 1));
        /*
        idk
        []something
        */
        println!("expected: {expected_doc_cursor:?}\ngot: {doc_cursor:?}");
        assert!(doc_cursor == expected_doc_cursor);
    }
    #[test]
    fn document_cursor_position_works_when_rope_cursor_head_and_anchor_different_and_on_different_lines(){
        let text = Rope::from("idk\nsomething");
        let rope_cursor = RopeCursor{anchor: 2, head: 5, stored_line_position: 1};  //idk[k\ns]omething
        let doc_cursor = Document::rope_cursor_position_to_document_cursor_position(rope_cursor, text.slice(..));
        let expected_doc_cursor = DocumentCursor::new(Position::new(1, 1), Position::new(2, 0));
        /*
        id[k
        s]omething
        */
        println!("expected: {expected_doc_cursor:?}\ngot: {doc_cursor:?}");
        assert!(doc_cursor == expected_doc_cursor);
    }

//CLEAR ROPE CURSORS EXCEPT MAIN
    #[test]
    fn clear_cursors_except_main_works(){
        let mut cursors = vec![RopeCursor::default(), RopeCursor::default(), RopeCursor::default()];
        Document::clear_cursors_except_main(&mut cursors);
        assert!(cursors.get(0).is_some());
        assert!(cursors.get(1).is_none());
    }

//SET ROPE CURSOR POSITION
    #[test]
    fn set_rope_cursor_position_works_when_desired_position_is_inside_doc_boundaries(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor::default();    //[]idk\nsomething\nelse
        let expected_rope_cursor = RopeCursor{anchor: 14, head: 14, stored_line_position: 0};   //idk\nsomething\n[]else
        let line_number: usize = 2;//3;
        rope_cursor = Document::set_rope_cursor_position_from_line_number(rope_cursor, line_number, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn set_rope_cursor_position_should_do_nothing_when_desired_line_number_is_greater_than_doc_length(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor::default();    //[]idk\nsomething\nelse
        let expected_rope_cursor = RopeCursor::default();   //[]idk\nsomething\nelse
        let line_number: usize = 5;//6;
        rope_cursor = Document::set_rope_cursor_position_from_line_number(rope_cursor, line_number, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn set_rope_cursor_position_restricts_cursor_to_line_end_when_cursor_stored_line_position_is_greater_than_line_width(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor{anchor: 13, head: 13, stored_line_position: 9};    //idk\nsomething[]\nelse
        let expected_rope_cursor = RopeCursor{anchor: 3, head: 3, stored_line_position: 9}; //idk[]\nsomething\nelse
        let line_number: usize = 0;//1;
        rope_cursor = Document::set_rope_cursor_position_from_line_number(rope_cursor, line_number, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }

//add cursor on line above
    //#[test]
    //fn add_cursor_on_line_above_works(){
    //    let mut doc = Document::default();
    //    doc.lines = vec!["idk".to_string(), "something".to_string()];
    //
    //    let cursor = doc.cursors.get_mut(0).unwrap();
    //    let position = Position::new(9, 1);
    //    *cursor = Document::set_cursor_position(cursor, position, &doc.lines).unwrap();
    //    doc.add_cursor_on_line_above();
    //    println!("{:?}", doc.cursors);
    //    assert!(doc.cursors[0].head == Position::new(9, 1));
    //    assert!(doc.cursors[0].anchor == Position::new(9, 1));
    //    assert!(doc.cursors[1].head == Position::new(3, 0));
    //    assert!(doc.cursors[1].anchor == Position::new(3, 0));
    //}
    //#[test]
    //fn add_cursor_on_line_above_works_after_adding_cursor_on_line_below(){
    //    // loop through cursors. save cursor with lowest y. add one above that
    //    assert!(false);
    //}
    //add cursor on line below
    //#[test]
    //fn add_cursor_on_line_below_works(){
    //    assert!(false);
    //}
    //#[test]
    //fn add_cursor_on_line_below_works_after_adding_cursor_on_line_above(){
    //    assert!(false);
    //}

// ENTER
    #[test]
    fn enter_works(){
        let text = Rope::from("idk\nsomething");
        let mut rope_cursor = RopeCursor{anchor: 13, head: 13, stored_line_position: 9};    //idk\nsomething[]
        let expected_rope_cursor = RopeCursor{anchor: 14, head: 14, stored_line_position: 0}; //idk\nsomething\n[]
        let mut new_text = Rope::new();
        let expected_text = Rope::from("idk\nsomething\n");
        (rope_cursor, new_text) = Document::insert_char_at_cursor(rope_cursor, text.slice(..), '\n');
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        println!("{:?} : {:?}", text, new_text);
        assert!(rope_cursor == expected_rope_cursor);
        assert!(new_text == expected_text);
    }
// AUTO-INDENT
    //#[test]
    //fn auto_indent_works(){
    //    assert!(false);
    //}
    
//INSERT CHAR
    #[test]
    fn insert_char_works(){
        let text = Rope::from("idk\nsomething\n");
        let mut rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0};    //idk\n[]something\n
        let expected_rope_cursor = RopeCursor{anchor: 5, head: 5, stored_line_position: 1}; //idk\nx[]something\n
        let mut new_text = Rope::new();
        let expected_text = Rope::from("idk\nxsomething\n");
        (rope_cursor, new_text) = Document::insert_char_at_cursor(rope_cursor, text.slice(..), 'x');
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        println!("{:?} : {:?}", text, new_text);
        assert!(rope_cursor == expected_rope_cursor);
        assert!(new_text == expected_text);
    }

//INSERT SELECTION
    //#[test]
    //fn single_cursor_insert_single_line_selection_works(){
    //    assert!(false);
    //}
    //#[test]
    //fn single_cursor_insert_multi_line_selection_works(){
    //    assert!(false);
    //}

//TAB
    //#[test]
    //fn single_cursor_insert_tab_works(){
    //    let mut doc = Document::default();
//
    //    doc.tab();
    //    let mut exptected_line = String::new();
    //    for _ in 0..TAB_WIDTH{
    //        exptected_line.push(' ');
    //    }
    //    assert!(doc.lines == vec![exptected_line]);
    //    assert!(doc.cursors.last().unwrap().head.x() == TAB_WIDTH);
    //    assert!(doc.cursors.last().unwrap().head.y() == 0);
    //    assert!(doc.cursors.last().unwrap().anchor.x() == TAB_WIDTH);
    //    assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    //}

//DELETE
    #[test]
    fn delete_works(){
        let text = Rope::from("idk\nsomething\n");
        let rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0};    //idk\n[]something\n
        let expected_rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0}; //idk\n[]omething\n
        let mut new_text = Rope::new();
        let expected_text = Rope::from("idk\nomething\n");
        new_text = Document::delete_at_cursor(rope_cursor.clone(), text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        println!("{:?} : {:?}", text, new_text);
        assert!(rope_cursor == expected_rope_cursor);
        assert!(new_text == expected_text);
    }
    //#[test]
    //fn single_cursor_delete_at_end_of_line_appends_next_line_to_current(){
    //    let mut doc = Document::default();
    //    doc.lines = vec!["idk".to_string(), "something".to_string()];
//
    //    let cursor = doc.cursors.get_mut(0).unwrap();
    //    let position = Position::new(3, 0);
    //    *cursor = Document::set_cursor_position(cursor, position, &doc.lines).unwrap();
    //    Document::delete_at_cursor(cursor, &mut doc.lines, &mut doc.modified);
    //    assert!(doc.lines == vec!["idksomething".to_string()]);
    //    assert!(cursor.head.x() == 3);
    //    assert!(cursor.head.y() == 0);
    //    assert!(cursor.anchor.x() == 3);
    //    assert!(cursor.anchor.y() == 0);
    //}
    //#[test]
    //fn single_cursor_delete_removes_selection(){
    //    assert!(false);
    //}
    #[test]
    fn delete_at_end_of_file_does_nothing(){
        let text = Rope::from("idk\nsomething\n");
        let rope_cursor = RopeCursor{anchor: 14, head: 14, stored_line_position: 0};    //idk\nsomething\n[]
        let expected_rope_cursor = RopeCursor{anchor: 14, head: 14, stored_line_position: 0}; //idk\nsomething\n[]
        let mut new_text = Rope::new();
        let expected_text = Rope::from("idk\nsomething\n");
        new_text = Document::delete_at_cursor(rope_cursor.clone(), text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        println!("{:?} : {:?}", text, new_text);
        assert!(rope_cursor == expected_rope_cursor);
        assert!(new_text == expected_text);
    }
    
//BACKSPACE
    //#[test]
    //fn single_cursor_backspace_removes_previous_character(){
    //    let mut doc = Document::default();
    //    doc.lines = vec!["idk".to_string()];
//
    //    let cursor = doc.cursors.get_mut(0).unwrap();
    //    let position = Position::new(1, 0);
    //    *cursor = Document::set_cursor_position(cursor, position, &doc.lines).unwrap();
    //    Document::backspace_at_cursor(cursor, &mut doc.lines, &mut doc.modified);
    //    println!("{:?}", doc.lines);
    //    assert!(doc.lines == vec!["dk".to_string()]);
    //    println!("{:?}", cursor.head);
    //    assert!(cursor.head.x() == 0);
    //    assert!(cursor.head.y() == 0);
    //    assert!(cursor.anchor.x() == 0);
    //    assert!(cursor.anchor.y() == 0);
    //}
    //#[test]
    //fn single_cursor_backspace_at_start_of_line_appends_current_line_to_end_of_previous_line(){
    //    let mut doc = Document::default();
    //    doc.lines = vec!["idk".to_string(), "something".to_string()];
//
    //    let cursor = doc.cursors.get_mut(0).unwrap();
    //    let position = Position::new(0, 1);
    //    *cursor = Document::set_cursor_position(cursor, position, &doc.lines).unwrap();
    //    Document::backspace_at_cursor(cursor, &mut doc.lines, &mut doc.modified);
    //    println!("{:?}", doc.lines);
    //    assert!(doc.lines == vec!["idksomething".to_string()]);
    //    println!("{:?}", cursor.head);
    //    assert!(cursor.head.x() == 3);
    //    assert!(cursor.head.y() == 0);
    //    assert!(cursor.anchor.x() == 3);
    //    assert!(cursor.anchor.y() == 0);
    //}
    //#[test]
    //fn single_cursor_backspace_removes_previous_tab(){
    //    let mut doc = Document::default();
    //    let mut line = String::new();
    //    for _ in 0..TAB_WIDTH{
    //        line.push(' ');
    //    }
    //    line.push_str("something");
    //    doc.lines = vec![line];
//
    //    let cursor = doc.cursors.get_mut(0).unwrap();
    //    let position = Position::new(TAB_WIDTH, 0);
    //    *cursor = Document::set_cursor_position(cursor, position, &doc.lines).unwrap();
    //    Document::backspace_at_cursor(cursor, &mut doc.lines, &mut doc.modified);
    //    println!("{:?}", doc.lines);
    //    assert!(doc.lines == vec!["something".to_string()]);
    //    println!("{:?}", cursor.head);
    //    assert!(cursor.head.x() == 0);
    //    assert!(cursor.head.y() == 0);
    //    assert!(cursor.anchor.x() == 0);
    //    assert!(cursor.anchor.y() == 0);
    //}
    
//MOVE CURSOR LEFT
    #[test]
    fn move_cursor_left_at_document_start_does_not_move_cursor(){
        let text = Rope::from("idk\nsomething\nelse");  //TODO: set up a better text to use, specific to move left
        let mut rope_cursor = RopeCursor::default();
        let expected_rope_cursor = RopeCursor::default();
        rope_cursor = Document::move_cursor_left(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn move_left_works(){
        let text = Rope::from("idk\nsomething\nelse");  //TODO: set up a better text to use, specific to move left
        let mut rope_cursor = RopeCursor{anchor: 2, head: 2, stored_line_position: 2};
        let expected_rope_cursor = RopeCursor{anchor: 1, head: 1, stored_line_position: 1};
        rope_cursor = Document::move_cursor_left(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn move_cursor_left_at_start_of_line_resets_stored_line_position(){
        let text = Rope::from("idk\nsomething\nelse");  //TODO: set up a better text to use, specific to move left
        let mut rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0};  //idk\n[]something\nelse
        let expected_rope_cursor = RopeCursor{anchor: 3, head: 3, stored_line_position: 3}; //idk[]\nsomething\nelse
        rope_cursor = Document::move_cursor_left(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    
//MOVE CURSOR UP
    #[test]
    fn move_cursor_up_at_document_start_does_not_move_cursor(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor::default();
        let expected_rope_cursor = RopeCursor::default();
        rope_cursor = Document::move_cursor_up(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn move_cursor_up_works_when_moving_to_shorter_line(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor{anchor: 13, head: 13, stored_line_position: 9};  //idk\nsomething[]\nelse
        let expected_rope_cursor = RopeCursor{anchor: 3, head: 3, stored_line_position: 9};   //idk[]\nsomething\nelse  //should maintain previous stored_line_position
        rope_cursor = Document::move_cursor_up(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn move_cursor_up_works_when_moving_to_longer_line(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor{anchor: 18, head: 18, stored_line_position: 4};    //idk\nsomething\nelse[]
        let expected_rope_cursor = RopeCursor{anchor: 8, head: 8, stored_line_position: 4}; //idk\nsome[]thing\nelse
        rope_cursor = Document::move_cursor_up(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }

//MOVE CURSOR RIGHT
    #[test]
    fn move_cursor_right_at_document_end_does_not_move_cursor(){
        let text = Rope::from("012\n");
        let mut rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0};  //012\n[]
        let expected_rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0}; //012\n[]
        rope_cursor = Document::move_cursor_right(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);

    }
    #[test]
    fn move_cursor_right_works(){
        let text = Rope::from("012\n");
        let mut rope_cursor = RopeCursor::default();    //[]012\n
        let expected_rope_cursor = RopeCursor{anchor: 1, head: 1, stored_line_position: 1}; //0[]12\n
        rope_cursor = Document::move_cursor_right(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn move_cursor_right_at_end_of_line_resets_stored_line_position(){
        let text = Rope::from("012\n0");
        let mut rope_cursor = RopeCursor{anchor: 3, head: 3, stored_line_position: 3};  //012[]\n0
        let expected_rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0}; //012\n[]0
        rope_cursor = Document::move_cursor_right(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }

//MOVE CURSOR DOWN
    #[test]
    fn move_cursor_down_at_document_end_does_not_move_cursor(){
        let text = Rope::from("012\n");
        let mut rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0};  //012\n[]
        let expected_rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0}; //012\n[]
        rope_cursor = Document::move_cursor_down(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn move_cursor_down_works_when_moving_to_shorter_line(){
        let text = Rope::from("012\n0");
        let mut rope_cursor = RopeCursor{anchor: 3, head: 3, stored_line_position: 3};  //012[]\n0
        let expected_rope_cursor = RopeCursor{anchor: 5, head: 5, stored_line_position: 3}; //012\n0[]  //should maintain previous stored_line_position
        rope_cursor = Document::move_cursor_down(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn move_cursor_down_works_when_moving_to_longer_line(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor{anchor: 3, head: 3, stored_line_position: 3}; //idk[]\nsomething\nelse
        let expected_rope_cursor = RopeCursor{anchor: 7, head: 7, stored_line_position: 3}; //idk\nsom[]ething\nelse
        rope_cursor = Document::move_cursor_down(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }

//MOVE CURSOR PAGE UP
    #[test]
    fn move_cursor_page_up_works(){
        let text = Rope::from("idk\nsomething\nelse");
        let client_view = View{horizontal_start: 0, vertical_start: 0, width: 2, height: 2};
        let mut rope_cursor = RopeCursor{anchor: 6, head: 6, stored_line_position: 2};  //idk\nso[]mething\nelse
        let expected_rope_cursor = RopeCursor{anchor: 2, head: 2, stored_line_position: 2}; //id[]k\nsomething\nelse
        rope_cursor = Document::move_cursor_page_up(rope_cursor, text.slice(..), client_view);
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
//MOVE CURSOR PAGE DOWN
    #[test]
    fn move_cursor_page_down_works(){
        let text = Rope::from("idk\nsomething\nelse");
        let client_view = View{horizontal_start: 0, vertical_start: 0, width: 2, height: 2};
        let mut rope_cursor = RopeCursor::default();  //[]idk\nsomething\nelse
        let expected_rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0}; //idk\n[]something\nelse
        rope_cursor = Document::move_cursor_page_down(rope_cursor, text.slice(..), client_view);
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }

//MOVE CURSOR HOME
    #[test]
    fn move_cursor_home_moves_cursor_to_text_start_when_cursor_past_text_start(){
        let text = Rope::from("    idk\n");
        let mut rope_cursor = RopeCursor{anchor: 6, head: 6, stored_line_position: 6};  //    id[]k\n
        let expected_rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 4}; //    []idk\n
        rope_cursor = Document::move_cursor_home(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn move_cursor_home_moves_cursor_to_line_start_when_cursor_at_text_start(){
        let text = Rope::from("    idk\n");
        let mut rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 4};  //    []idk\n
        let expected_rope_cursor = RopeCursor::default();   //[]    idk\n
        rope_cursor = Document::move_cursor_home(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn move_cursor_home_moves_cursor_to_text_start_when_cursor_before_text_start(){
        let text = Rope::from("    idk\n");
        let mut rope_cursor = RopeCursor{anchor: 1, head: 1, stored_line_position: 1};  // []   idk\n
        let expected_rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 4}; //    []idk\n
        rope_cursor = Document::move_cursor_home(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    //TODO: should there be another test to verify stored line position functionality in multiline texts?

//MOVE CURSOR END
    #[test]
    fn move_cursor_end_moves_cursor_to_line_end(){
        let text = Rope::from("idk\n");
        let mut rope_cursor = RopeCursor::default();    //[]idk\n
        let expected_rope_cursor = RopeCursor{anchor: 3, head: 3, stored_line_position: 3}; //idk[]\n
        rope_cursor = Document::move_cursor_end(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }

//MOVE CURSOR DOC START
    #[test]
    fn move_cursor_doc_start_works(){
        let mut rope_cursor = RopeCursor{anchor: 12, head: 12, stored_line_position: 12};
        let expected_rope_cursor = RopeCursor::default();
        rope_cursor = Document::move_cursor_document_start(rope_cursor);
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
//MOVE CURSOR DOC END
    #[test]
    fn move_cursor_document_end_works(){
        let text = Rope::from("idk\nsome\nshit");
        let mut rope_cursor = RopeCursor::default();    //[]idk\nsome\nshit
        let expected_rope_cursor = RopeCursor{anchor: 13, head: 13, stored_line_position: 4};   //idk\nsome\nshit[]
        rope_cursor = Document::move_cursor_document_end(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }

//EXTEND SELECTION RIGHT
    #[test]
    fn extend_selection_right_at_document_end_does_not_extend_selection(){
        let text = Rope::from("012\n");
        let mut rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0};  //012\n[]
        let expected_rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0}; //012\n[]
        rope_cursor = Document::extend_selection_right(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);

    }
    #[test]
    fn extend_selection_right_works(){
        let text = Rope::from("012\n");
        let mut rope_cursor = RopeCursor::default();    //[]012\n
        let expected_rope_cursor = RopeCursor{anchor: 0, head: 1, stored_line_position: 1}; //[0]12\n
        rope_cursor = Document::extend_selection_right(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn extend_selection_right_at_end_of_line_resets_stored_line_position(){
        let text = Rope::from("012\n0");
        let mut rope_cursor = RopeCursor{anchor: 3, head: 3, stored_line_position: 3};  //012[]\n0
        let expected_rope_cursor = RopeCursor{anchor: 3, head: 4, stored_line_position: 0}; //012[\n]0
        rope_cursor = Document::extend_selection_right(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    //TODO: test if selection extended left, extend selection right reduces selection

//EXTEND SELECTION LEFT
    #[test]
    fn extend_selection_left_at_document_start_does_not_extend_selection(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor::default();
        let expected_rope_cursor = RopeCursor::default();
        rope_cursor = Document::extend_selection_left(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn extend_selection_left_works(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor{anchor: 2, head: 2, stored_line_position: 2};  //id[]k\nsomthing\nelse
        let expected_rope_cursor = RopeCursor{anchor: 2, head: 1, stored_line_position: 1}; //i]d[k\nsomething\nelse
        rope_cursor = Document::extend_selection_left(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn extend_selection_left_at_start_of_line_resets_stored_line_position(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0};  //idk\n[]something\nelse
        let expected_rope_cursor = RopeCursor{anchor: 4, head: 3, stored_line_position: 3}; //idk]\n[something\nelse
        rope_cursor = Document::extend_selection_left(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    //TODO: test if selection extended right, extend selection left reduces selection

//EXTEND SELECTION UP
    #[test]
    fn extend_selection_up_at_document_start_does_not_extend_selection(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor::default();
        let expected_rope_cursor = RopeCursor::default();
        rope_cursor = Document::extend_selection_up(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn extend_selection_up_works_when_moving_to_shorter_line(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor{anchor: 13, head: 13, stored_line_position: 9};  //idk\nsomething[]\nelse
        let expected_rope_cursor = RopeCursor{anchor: 13, head: 3, stored_line_position: 9};   //idk]\nsomething[\nelse  //should maintain previous stored_line_position
        rope_cursor = Document::extend_selection_up(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn extend_selection_up_works_when_moving_to_longer_line(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor{anchor: 18, head: 18, stored_line_position: 4};    //idk\nsomething\nelse[]
        let expected_rope_cursor = RopeCursor{anchor: 18, head: 8, stored_line_position: 4}; //idk\nsome]thing\nelse[
        rope_cursor = Document::extend_selection_up(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    
//EXTEND SELECTION DOWN
    #[test]
    fn extend_selection_down_at_document_end_does_not_extend_selection(){
        let text = Rope::from("012\n");
        let mut rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0};  //012\n[]
        let expected_rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 0}; //012\n[]
        rope_cursor = Document::extend_selection_down(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn extend_selection_down_works_when_moving_to_shorter_line(){
        let text = Rope::from("012\n0");
        let mut rope_cursor = RopeCursor{anchor: 3, head: 3, stored_line_position: 3};  //012[]\n0
        let expected_rope_cursor = RopeCursor{anchor: 3, head: 5, stored_line_position: 3}; //012[\n0]  //should maintain previous stored_line_position
        rope_cursor = Document::extend_selection_down(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn extend_selection_down_works_when_moving_to_longer_line(){
        let text = Rope::from("idk\nsomething\nelse");
        let mut rope_cursor = RopeCursor{anchor: 3, head: 3, stored_line_position: 3}; //idk[]\nsomething\nelse
        let expected_rope_cursor = RopeCursor{anchor: 3, head: 7, stored_line_position: 3}; //idk[\nsom]ething\nelse
        rope_cursor = Document::extend_selection_down(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    
//EXTEND SELECTION HOME
    #[test]
    fn extend_selection_home_moves_cursor_to_text_start_when_cursor_past_text_start(){
        let text = Rope::from("    idk\n");
        let mut rope_cursor = RopeCursor{anchor: 6, head: 6, stored_line_position: 6};  //    id[]k\n
        let expected_rope_cursor = RopeCursor{anchor: 6, head: 4, stored_line_position: 4}; //    ]id[k\n
        rope_cursor = Document::extend_selection_home(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn extend_selection_home_moves_cursor_to_line_start_when_cursor_at_text_start(){
        let text = Rope::from("    idk\n");
        let mut rope_cursor = RopeCursor{anchor: 4, head: 4, stored_line_position: 4};  //    []idk\n
        let expected_rope_cursor = RopeCursor{anchor: 4, head: 0, stored_line_position: 0};   //]    [idk\n
        rope_cursor = Document::extend_selection_home(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }
    #[test]
    fn extend_selection_home_moves_cursor_to_text_start_when_cursor_before_text_start(){
        let text = Rope::from("    idk\n");
        let mut rope_cursor = RopeCursor{anchor: 1, head: 1, stored_line_position: 1};  // []   idk\n
        let expected_rope_cursor = RopeCursor{anchor: 1, head: 4, stored_line_position: 4}; // [   ]idk\n
        rope_cursor = Document::extend_selection_home(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }

//EXTEND SELECTION END
    #[test]
    fn extend_selection_end_moves_cursor_to_line_end(){
        let text = Rope::from("idk\n");
        let mut rope_cursor = RopeCursor::default();    //[]idk\n
        let expected_rope_cursor = RopeCursor{anchor: 0, head: 3, stored_line_position: 3}; //[idk]\n
        rope_cursor = Document::extend_selection_end(rope_cursor, text.slice(..));
        println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
        assert!(rope_cursor == expected_rope_cursor);
    }

//extend selection page up
//extend selection page down
    
//COLLAPSE SELECTION CURSOR
    #[test]
    fn collapse_selection_cursor_works_when_head_less_than_anchor(){
        assert!(false);
    }
    #[test]
    fn collapse_selection_cursor_works_when_head_greater_than_anchor(){
        assert!(false);
    }

//goto

//scroll client view down
//scroll client view left
//scroll client view right
//scroll client view up
//scroll view following cursor

//set client view size (does this need testing?)
//get client view text
    #[test]
    fn get_client_view_text_works(){
        let mut doc = Document::default();
        doc.text = Rope::from("idk\nsomething\nelse\n");
        doc.set_client_view_size(2, 2);
        println!("{:?}", doc.get_client_view_text());
        assert!(doc.get_client_view_text() == String::from("id\nso\n"));
    }
//get client view line numbers
//get client cursor positions
//}
