use crate::{Position, View};
use std::fs;
use std::io::{Error, Write};
use std::path::PathBuf;
use unicode_segmentation::UnicodeSegmentation;

// tab keypress inserts the number of spaces specified in TAB_WIDTH into the focused document
pub const TAB_WIDTH: usize = 4;



#[derive(Default, Clone, Debug)]
struct Cursor{
    anchor: Position,
    head: Position,
    stored_line_position: usize,
}
impl Cursor{
    pub fn new(anchor: Position, head: Position) -> Self{
        Self{anchor, head, stored_line_position: head.x}
    }
    pub fn set_both_x(&mut self, x: usize){
        self.anchor.set_x(x);
        self.head.set_x(x);
    }
    pub fn set_both_y(&mut self, y: usize){
        self.anchor.set_y(y);
        self.head.set_y(y);
    }
}
pub struct Document{
    lines: Vec<String>,
    file_name: Option<String>, //TODO: should no longer need to be optional. we are enforcing a doc being open in client
    modified: bool,
    cursors: Vec<Cursor>,
    client_view: View,
}
impl Default for Document{
    fn default() -> Self {
        Self{
            lines: vec![String::new()],
            file_name: None,
            modified: false,
            cursors: vec![Cursor::default()],
            client_view: View::default(),
        }
    }
}
impl Document{
    pub fn open(path: &PathBuf) -> Result<Self, std::io::Error>{
        let file_content = fs::read_to_string(path)?;
        let mut lines = Vec::new();
        if file_content.is_empty(){
            lines.push(String::new()); //prevents crash when inserting into empty document
        }else{
            for line in file_content.lines(){
                lines.push(line.to_string());
            }
        }
    
        Ok(Self{
            lines,
            file_name: Some(path.to_string_lossy().to_string()),
            modified: false,
            cursors: vec![Cursor::default()],
            client_view: View::default(),
        })
    }

    pub fn file_name(&self) -> Option<String>{
        self.file_name.clone()
    }
    pub fn set_file_name(&mut self, file_name: Option<String>){
        self.file_name = file_name;
    }

    /// Position is 0 based. 
    /// When using cursor_position as an index, just use cursor_position. 
    /// When using cursor_position for line number, use cursor_position.saturating_sub(1). 
    pub fn cursor_position(&self) -> Position{
        let cursor = self.cursors.last().unwrap();
        cursor.head
    }
    pub fn cursor_positions(&self) -> Vec<Position>{
        let mut positions = Vec::new();
        for cursor in &self.cursors{
            positions.push(cursor.head);
        }
        positions
    }
    fn clear_cursors_except_main(cursors: &mut Vec<Cursor>){
        for x in (0..cursors.len()).rev(){
            if x != 0{
                cursors.pop();
            }
        }
    }
    fn set_cursor_position(cursor: &mut Cursor, position: Position, lines: &Vec<String>){
        match lines.get(position.y()){
            Some(line) => {
                let line_width = line.graphemes(true).count();
                if position.x() < line_width
                || position.x() == line_width{
                    cursor.anchor = position;
                    cursor.head = position;
                    cursor.stored_line_position = position.x();
                }else{
                    let new_pos = Position::new(line_width, position.y());
                    cursor.anchor = new_pos;
                    cursor.head = new_pos;
                    cursor.stored_line_position = new_pos.x();
                }
            }
            None => {}
        }
    }

    pub fn add_cursor_on_line_above(&mut self){
        self.cursors.push(
            Cursor::new(
                Position::new(self.cursors.last().unwrap().head.x, self.cursors.last().unwrap().head.y.saturating_sub(1)),
                Position::new(self.cursors.last().unwrap().head.x, self.cursors.last().unwrap().head.y.saturating_sub(1))
            )
        );

        //unwrapping because this is guaranteed to have a cursor because one was just added
        let cursor = self.cursors.last_mut().unwrap();
        Document::clamp_cursor_to_line_end(cursor, &self.lines);
    }
    pub fn add_cursor_on_line_below(&mut self){

    }

    pub fn enter(&mut self){        
        for cursor in self.cursors.iter_mut(){
            Document::enter_at_cursor(cursor, &mut self.lines, &mut self.modified);
        }
    }
    // auto indent doesn't work correctly if previous line has only whitespace characters
    // also doesn't auto indent for first line of function bodies, because function declaration
    // is at lower indentation level
    fn enter_at_cursor(cursor: &mut Cursor, lines: &mut Vec<String>, modified: &mut bool){
        *modified = true;
        
        match lines.get_mut(cursor.head.y){
            Some(line) => {
                let start_of_line = get_first_non_whitespace_character_index(line);
                let mut modified_current_line: String = String::new();
                let mut new_line: String = String::new();
                for (index, grapheme) in line[..].graphemes(true).enumerate(){
                    if index < cursor.head.x{
                        modified_current_line.push_str(grapheme);
                    }
                    else{
                        new_line.push_str(grapheme);
                    }
                }
                *line = modified_current_line;
                lines.insert(cursor.head.y.saturating_add(1), new_line);
                Document::move_cursor_right(cursor, &lines);
                // auto indent
                if start_of_line != 0{
                    for _ in 0..start_of_line{
                        Document::insert_char_at_cursor(' ', cursor, lines, modified);
                    }
                }
            }
            None => panic!("No line at cursor position. This should be impossible")
        }
    }

    pub fn insert_char(&mut self, c: char){
        for cursor in self.cursors.iter_mut(){
            Document::insert_char_at_cursor(c, cursor, &mut self.lines, &mut self.modified);
        }
    }
    fn insert_char_at_cursor(c: char, cursor: &mut Cursor, lines: &mut Vec<String>, modified: &mut bool){
        *modified = true;
        
        match lines.get_mut(cursor.head.y){
            Some(line) => {
                line.insert(cursor.head.x, c);
                Document::move_cursor_right(cursor, &lines);
            }
            None => panic!("No line at cursor position. This should be impossible")
        };
    }

    pub fn tab(&mut self){
        for cursor in self.cursors.iter_mut(){
            let tab_distance = distance_to_next_multiple_of_tab_width(&cursor);
            let modified_tab_width = if tab_distance > 0 && tab_distance < TAB_WIDTH{
                tab_distance
            }else{
                TAB_WIDTH
            };
            for _ in 0..modified_tab_width{
                Document::insert_char_at_cursor(' ', cursor, &mut self.lines, &mut self.modified);
            }
        }
    }
    //
    //fn tab_at_cursor(){}?
    //

    pub fn delete(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::delete_at_cursor(cursor, &mut self.lines, &mut self.modified);
        }
    }
    fn delete_at_cursor(cursor: &mut Cursor, lines: &mut Vec<String>, modified: &mut bool){
        *modified = true;
        
        if cursor.head.x == cursor.anchor.x && cursor.head.y == cursor.anchor.y{
            match (Document::cursor_on_last_line(cursor, lines), Document::cursor_at_end_of_line(cursor, lines)){
                (true, true) => {/*do nothing*/}
                (_, false) => {
                    match lines.get_mut(cursor.head.y){
                        Some(line) => {
                            if cursor.head.x < line.graphemes(true).count(){
                                line.remove(cursor.head.x);
                            }
                        }
                        None => panic!("No line at cursor position. This should be impossible")
                    };
                }
                (false, true) => {
                    let next_line = lines.remove(cursor.head.y.saturating_add(1));
                    match lines.get_mut(cursor.head.y){
                        Some(line) => {
                            line.push_str(&next_line);
                        }
                        None => panic!("No line at cursor position. This should be impossible")
                    };
                }
            }
        }else{
            // delete selection
        }
    }

    fn cursor_on_last_line(cursor: &Cursor, lines: &Vec<String>) -> bool{
        cursor.head.y.saturating_add(1) == lines.len()
    }
    fn cursor_at_end_of_line(cursor: &Cursor, lines: &Vec<String>) -> bool{
        match lines.get(cursor.head.y){
            Some(line) => cursor.head.x == line.graphemes(true).count(),
            None => panic!("No line at cursor position. This should be impossible")
        }
    }

    pub fn backspace(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::backspace_at_cursor(cursor, &mut self.lines, &mut self.modified);
        }
    }
    fn backspace_at_cursor(cursor: &mut Cursor, lines: &mut Vec<String>, modified: &mut bool){
        if cursor.head.x >= TAB_WIDTH
            // handles case where user adds a space after a tab, and wants to delete only the space
            && cursor.head.x % TAB_WIDTH == 0
            // if previous 4 chars are spaces, delete 4. otherwise, use default behavior
            && slice_is_all_spaces(
                match lines.get(cursor.head.y){
                    Some(line) => line,
                    None => panic!("No line at cursor position. This should be impossible")
                },
                cursor.head.x - TAB_WIDTH, 
                cursor.head.x,
            ){
                for _ in 0..TAB_WIDTH{
                    Document::move_cursor_left(cursor, lines);
                    Document::delete_at_cursor(cursor, lines, modified);
                }
            }
            else if cursor.head.x > 0{
                Document::move_cursor_left(cursor, lines);
                Document::delete_at_cursor(cursor, lines, modified);
            }
            else if cursor.head.x == 0 && cursor.head.y > 0{
                Document::move_cursor_left(cursor, lines);
                Document::delete_at_cursor(cursor, lines, modified);
            }
    }

    pub fn move_cursors_up(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::move_cursor_up(cursor, &self.lines);
        }
    }
    fn move_cursor_up(cursor: &mut Cursor, lines: &Vec<String>){
        cursor.set_both_y(cursor.head.y.saturating_sub(1));
        Document::clamp_cursor_to_line_end(cursor, lines)
    }

    pub fn move_cursors_down(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::move_cursor_down(cursor, &self.lines);
        }
    }
    fn move_cursor_down(cursor: &mut Cursor, lines: &Vec<String>){
        let document_length = lines.len();
        if cursor.head.y.saturating_add(1) < document_length{
            cursor.set_both_y(cursor.head.y.saturating_add(1));
        }
        Document::clamp_cursor_to_line_end(cursor, lines);
    }

    pub fn move_cursors_right(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::move_cursor_right(cursor, &self.lines);
        }
    }
    fn move_cursor_right(cursor: &mut Cursor, lines: &Vec<String>){
        match lines.get(cursor.head.y){
            Some(line) => {
                if cursor.head.x < line.graphemes(true).count(){
                    cursor.set_both_x(cursor.head.x.saturating_add(1));
                }
                //move cursor to next line start
                else if cursor.head.y < lines.len().saturating_sub(1){
                    cursor.anchor.y = cursor.anchor.y.saturating_add(1);
                    cursor.head.y = cursor.head.y.saturating_add(1);
                    cursor.set_both_x(0);
                }
                cursor.stored_line_position = cursor.head.x;
            }
            None => panic!("No line at cursor position. This should be impossible.")
        }
    }

    pub fn move_cursors_left(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::move_cursor_left(cursor, &self.lines);
        }
    }
    fn move_cursor_left(cursor: &mut Cursor, lines: &Vec<String>){
        if cursor.head.x > 0{
            cursor.anchor.x = cursor.anchor.x.saturating_sub(1);
            cursor.head.x = cursor.head.x.saturating_sub(1);
        }
        //move cursor to previous line end
        else if cursor.head.y > 0{
            cursor.anchor.y = cursor.anchor.y.saturating_sub(1);
            cursor.head.y = cursor.head.y.saturating_sub(1);

            match lines.get(cursor.head.y){
                Some(line) => {
                    cursor.set_both_x(line.graphemes(true).count());
                }
                None => panic!("No line at cursor position. This should be impossible")
            };
        }
        cursor.stored_line_position = cursor.head.x;
    }

    pub fn move_cursors_page_up(&mut self){
        for cursor in self.cursors.iter_mut(){
            //cursor.set_both_y(
            //    if cursor.head.y >= self.client_view.height{
            //        cursor.head.y.saturating_sub(self.client_view.height - 1)
            //    }else{0}
            //);
            //Document::clamp_cursor_to_line_end(cursor, &self.lines);
            Document::move_cursor_page_up(cursor, &self.client_view, &self.lines);
        }
    }
    fn move_cursor_page_up(cursor: &mut Cursor, client_view: &View, lines: &Vec<String>){
        cursor.set_both_y(
            if cursor.head.y >= client_view.height{
                cursor.head.y.saturating_sub(client_view.height - 1)
            }else{0}
        );
        Document::clamp_cursor_to_line_end(cursor, lines);
    }

    pub fn move_cursors_page_down(&mut self){
        //let document_length = self.lines.len();
        for cursor in self.cursors.iter_mut(){
            //cursor.set_both_y(
            //    if cursor.head.y.saturating_add(self.client_view.height) <= document_length{
            //        cursor.head.y.saturating_add(self.client_view.height - 1)
            //    }else{
            //        document_length.saturating_sub(1)
            //    }
            //);
            //Document::clamp_cursor_to_line_end(cursor, &self.lines);
            Document::move_cursor_page_down(cursor, &self.client_view, &self.lines);
        }
    }
    fn move_cursor_page_down(cursor: &mut Cursor, client_view: &View, lines: &Vec<String>){
        let document_length = lines.len();
        cursor.set_both_y(
            if cursor.head.y.saturating_add(client_view.height) <= document_length{
                cursor.head.y.saturating_add(client_view.height - 1)
            }else{
                document_length.saturating_sub(1)
            }
        );
        Document::clamp_cursor_to_line_end(cursor, lines);
    }

    pub fn move_cursors_home(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::move_cursor_home(cursor, &self.lines);
        }
    }
    fn move_cursor_home(cursor: &mut Cursor, lines: &Vec<String>){
        match lines.get(cursor.head.y){
            Some(line) => {
                let start_of_line = get_first_non_whitespace_character_index(line);
                if cursor.head.x == start_of_line{
                    cursor.set_both_x(0);
                }else{
                    cursor.set_both_x(start_of_line);
                }
                cursor.stored_line_position = cursor.head.x;
            }
            None => panic!("No line at cursor position. This should be impossible")
        }
    }

    pub fn move_cursors_end(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::move_cursor_end(cursor, &self.lines);
        }
    }
    fn move_cursor_end(cursor: &mut Cursor, lines: &Vec<String>){
        match lines.get(cursor.head.y){
            Some(line) => {
                let line_width = line.graphemes(true).count();
                cursor.set_both_x(line_width);
                cursor.stored_line_position = cursor.head.x;
            }
            None => panic!("No line at cursor position. This should be impossible")
        };
    }

    pub fn move_cursors_document_start(&mut self){
        Document::clear_cursors_except_main(&mut self.cursors);
        match self.cursors.get_mut(0){
            Some(cursor) => {
                Document::move_cursor_document_start(cursor);
            }
            None => panic!("No cursor at 0 index. This should be impossible.")
        }
    }
    fn move_cursor_document_start(cursor: &mut Cursor){
        *cursor = Cursor::default();
        cursor.stored_line_position = cursor.head.x;
    }

    pub fn move_cursors_document_end(&mut self){
        Document::clear_cursors_except_main(&mut self.cursors);
        match self.cursors.get_mut(0){
            Some(cursor) => {
                Document::move_cursor_document_end(cursor, &self.lines);
            }
            None => panic!("No cursor at 0 index. This should be impossible.")
        }
    }
    fn move_cursor_document_end(cursor: &mut Cursor, lines: &Vec<String>){
        cursor.set_both_y(lines.len().saturating_sub(1));
        match lines.get(cursor.head.y){
            Some(line) => {
                let line_width = line.graphemes(true).count();
                cursor.set_both_x(line_width);
                cursor.stored_line_position = cursor.head.x;
            }
            None => panic!("No line at cursor position. This should be impossible.")
        };
    }

    pub fn extend_selection_right(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::extend_selection_right_at_cursor(cursor, &self.lines);
        }
    }
    fn extend_selection_right_at_cursor(cursor: &mut Cursor, lines: &Vec<String>){
        match lines.get(cursor.head.y){
            Some(line) => {
                let line_width = line.graphemes(true).count();
                if cursor.head.x < line_width{
                    cursor.head.x = cursor.head.x.saturating_add(1);
                }
                else if cursor.head.y < lines.len().saturating_sub(1){
                    cursor.head.y = cursor.head.y.saturating_add(1);
                    cursor.head.x = 0;
                }
                cursor.stored_line_position = cursor.head.x;
            }
            None => panic!("No line at cursor position. This should be impossible")
        };
    }

    pub fn extend_selection_left(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::extend_selection_left_at_cursor(cursor, &self.lines);
        }
    }
    fn extend_selection_left_at_cursor(cursor: &mut Cursor, lines: &Vec<String>){
        match lines.get(cursor.head.y){
            Some(line) => {
                if cursor.head.x > 0{
                    cursor.head.x = cursor.head.x.saturating_sub(1);
                }
                else if cursor.head.y > 0{
                    cursor.head.y = cursor.head.y.saturating_sub(1);
                    cursor.head.x = line.graphemes(true).count()
                }
                cursor.stored_line_position = cursor.head.x;
            }
            None => panic!("No line at cursor position. This should be impossible")
        };
    }

    pub fn extend_selection_up(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::extend_selection_up_at_cursor(cursor, &self.lines);
        }
    }
    fn extend_selection_up_at_cursor(cursor: &mut Cursor, lines: &Vec<String>){
        cursor.head.y = cursor.head.y.saturating_sub(1);
        Document::clamp_selection_cursor_to_line_end(cursor, lines);
    }

    pub fn extend_selection_down(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::extend_selection_down_at_cursor(cursor, &self.lines);
        }
    }
    fn extend_selection_down_at_cursor(cursor: &mut Cursor, lines: &Vec<String>){
        if cursor.head.y < lines.len().saturating_sub(1){
            cursor.head.y = cursor.head.y.saturating_add(1);
        }
        Document::clamp_selection_cursor_to_line_end(cursor, lines);
    }

    pub fn extend_selection_home(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::extend_selection_home_at_cursor(cursor, &self.lines);
        }
    }
    fn extend_selection_home_at_cursor(cursor: &mut Cursor, lines: &Vec<String>){
        match lines.get(cursor.head.y){
            Some(line) => {
                let start_of_line = get_first_non_whitespace_character_index(line);
                if cursor.head.x == start_of_line{
                    cursor.head.x = 0;
                }else{
                    cursor.head.x = start_of_line;
                }
                cursor.stored_line_position = cursor.head.x;
            }
            None => panic!("No line at cursor position. This should be impossible")
        };
    }

    pub fn extend_selection_end(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::extend_selection_end_at_cursor(cursor, &self.lines);
        }
    }
    fn extend_selection_end_at_cursor(cursor: &mut Cursor, lines: &Vec<String>){
        match lines.get(cursor.head.y){
            Some(line) => {
                let line_width = line.graphemes(true).count();
                cursor.head.x = line_width;
                cursor.stored_line_position = cursor.head.x;
            }
            None => panic!("No line at cursor position. This should be impossible")
        };
    }

    pub fn _extend_selection_page_up(&mut self){}

    pub fn _extend_selection_page_down(&mut self){}

    pub fn collapse_selection_cursors(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::collapse_selection_cursor(cursor);
        }
    }
    fn collapse_selection_cursor(cursor: &mut Cursor){
        if cursor.head.y == cursor.anchor.y{
            cursor.anchor.x = cursor.stored_line_position;
            cursor.head.x = cursor.stored_line_position;
        }else{
            cursor.anchor.x = cursor.stored_line_position;
            cursor.head.x = cursor.stored_line_position;
            cursor.anchor.y = cursor.head.y;
        }
    }

    fn clamp_cursor_to_line_end(cursor: &mut Cursor, lines: &Vec<String>){
        match lines.get(cursor.head.y){
            Some(line) => {
                let line_width = line.graphemes(true).count();
                cursor.set_both_x(
                    if cursor.head.x > line_width 
                    || cursor.stored_line_position > line_width{
                        line_width
                    }else{
                        cursor.stored_line_position
                    }
                );
            }
            None => panic!("No line at cursor position. This should be impossible")
        };
    }

    fn clamp_selection_cursor_to_line_end(cursor: &mut Cursor, lines: &Vec<String>){
        match lines.get(cursor.head.y){
            Some(line) => {
                let line_width = line.graphemes(true).count();
                if cursor.head.x > line_width
                || cursor.stored_line_position > line_width{
                    cursor.head.x = line_width;
                }else{
                    cursor.head.x = cursor.stored_line_position;
                }
            }
            None => panic!("No line at cursor position. This should be impossible")
        };
    }

    pub fn save(&mut self) -> Result<(), Error>{
        if let Some(file_name) = &self.file_name{ // does nothing if file_name is None
            let mut file = fs::File::create(file_name)?;
            
            for line in &self.lines {
                file.write_all(line.as_bytes())?;
                file.write_all(b"\n")?;
            }
            
            self.modified = false;
        }
        
        Ok(())
    }

    pub fn go_to(&mut self, line_number: usize){// -> Result<(), ()>{
        Document::clear_cursors_except_main(&mut self.cursors);
        match self.cursors.get_mut(0){
            Some(cursor) => {
                if line_number < self.lines.len(){
                    Document::set_cursor_position(cursor, Position::new(cursor.stored_line_position, line_number), &self.lines);
                }
            }
            None => panic!("No cursor at 0 index. This should be impossible.")
        }
    }

    pub fn lines(&self) -> &Vec<String>{
        &self.lines
    }

    pub fn lines_as_single_string(&self) -> String{
        let mut lines = String::new();
        for idk in self.lines.clone(){
            lines.push_str(format!("{}\n", idk).as_str())
        }
        lines
    }

    pub fn is_modified(&self) -> bool{
        self.modified
    }

    pub fn scroll_client_view_down(&mut self, amount: usize){
        if self.client_view.vertical_start + self.client_view.height + amount <= self.lines.len(){
            self.client_view.vertical_start = self.client_view.vertical_start.saturating_add(amount);
        }
    }
    pub fn scroll_client_view_left(&mut self, amount: usize){
        self.client_view.horizontal_start = self.client_view.horizontal_start.saturating_sub(amount);
    }
    pub fn scroll_client_view_right(&mut self, amount: usize){
        let mut longest = 0;
        for line in &self.lines{
            if line.len() > longest{
                longest = line.len();
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
        let cursor = self.cursors.last().expect("No cursor. This should be impossible");
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
        for (y, line) in self.lines.iter().enumerate(){
            let mut bounded_line = String::new();
            if y < self.client_view.vertical_start{}
            else if y > (self.client_view.height.saturating_sub(1) + self.client_view.vertical_start){/*can return early, because we're past our view */}
            else{
                for (x, char) in line.chars().enumerate(){
                    if x < self.client_view.horizontal_start{}
                    else if x > (self.client_view.width.saturating_sub(1) + self.client_view.horizontal_start){}
                    else{
                        bounded_line.push(char);
                    }
                }
                client_view_text.push_str(format!("{}\n", bounded_line).as_str());
            }
        }

        client_view_text
    }

    pub fn get_client_view_line_numbers(&self)-> String{
        let mut client_view_line_numbers = String::new();
        for (y, _) in self.lines.iter().enumerate(){
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
        for cursor in &self.cursors{
            if cursor.head.x >= self.client_view.horizontal_start
            && cursor.head.x < self.client_view.horizontal_start.saturating_add(self.client_view.width)
            && cursor.head.y >= self.client_view.vertical_start
            && cursor.head.y < self.client_view.vertical_start.saturating_add(self.client_view.height){
                positions.push(
                    Position{
                        x: cursor.head.x.saturating_sub(self.client_view.horizontal_start),
                        y: cursor.head.y.saturating_sub(self.client_view.vertical_start)
                    }
                );
            } 
        }
        positions
    }
}

fn get_first_non_whitespace_character_index(line: &str)-> usize{
    if line.is_empty(){
        return 0;
    }
    for (index, grapheme) in line[..].graphemes(true).enumerate(){
        if grapheme == " "{/*do nothing*/}
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

fn distance_to_next_multiple_of_tab_width(cursor: &Cursor) -> usize{
    if cursor.head.x % TAB_WIDTH != 0{
        TAB_WIDTH - (cursor.head.x % TAB_WIDTH)
    }else{
        0
    }
}





//#[cfg(test)]
//mod tests{
//    use crate::{document::Document, Position};

    #[test]
    fn clear_cursors_except_main_works(){
        let mut cursors = vec![Cursor::default(), Cursor::default(), Cursor::default()];
        Document::clear_cursors_except_main(&mut cursors);
        assert!(cursors.get(0).is_some());
        assert!(cursors.get(1).is_none());
    }

    //TODO: split into individual tests
    #[test]
    fn verify_set_cursor_position_behavior(){
        let mut doc = Document::default();
        doc.lines = vec!["1234".to_string(), "1".to_string(), "123".to_string()];

        // setting y inside doc should work
        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(0, 2);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y == 2);
        assert!(cursor.head.x == 0);
        // setting y past doc end does nothing
        let position = Position::new(0, 3);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y == 2);
        assert!(cursor.head.x == 0);
        // setting x past line end should restrict x to line end
        let position = Position::new(4, 2);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y == 2);
        assert!(cursor.head.x == 3);
    }

    //add cursor on line above
    #[test]
    fn add_cursor_on_line_above_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        let position = Position::new(9, 1);
        Document::set_cursor_position(doc.cursors.get_mut(0).unwrap(), position, &doc.lines);
        doc.add_cursor_on_line_above();
        println!("{:?}", doc.cursors);
        assert!(doc.cursors[0].head == Position::new(9, 1));
        assert!(doc.cursors[1].head == Position::new(3, 0));
    }
    #[test]
    fn add_cursor_on_line_above_works_after_adding_cursor_on_line_below(){
        assert!(false);
    }
    //add cursor on line below
    #[test]
    fn add_cursor_on_line_below_works(){
        assert!(false);
    }
    #[test]
    fn add_cursor_on_line_below_works_after_adding_cursor_on_line_above(){
        assert!(false);
    }

// ENTER
    #[test]
    fn single_cursor_enter_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::enter_at_cursor(cursor, &mut doc.lines, &mut doc.modified);
        println!("{:?}", doc.lines);
        assert!(doc.lines == vec!["".to_string(), "idk".to_string()]);
        assert!(cursor.head.x() == 0);
        assert!(cursor.head.y() == 1);
    }
// AUTO-INDENT
    #[test]
    fn auto_indent_works(){
        assert!(false);
    }
    
//INSERT CHAR
    #[test]
    fn single_cursor_insert_char_works(){
        let mut doc = Document::default();
        doc.lines = vec!["dk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::insert_char_at_cursor('i', cursor, &mut doc.lines, &mut doc.modified);
        assert!(doc.lines == vec!["idk".to_string()]);
        assert!(cursor.head.x() == 1);
        assert!(cursor.head.y() == 0);
    }

//INSERT SELECTION
    #[test]
    fn single_cursor_insert_single_line_selection_works(){
        assert!(false);
    }
    #[test]
    fn single_cursor_insert_multi_line_selection_works(){
        assert!(false);
    }

//TAB
    #[test]
    fn single_cursor_insert_tab_works(){
        let mut doc = Document::default();

        doc.tab();
        let mut exptected_line = String::new();
        for _ in 0..TAB_WIDTH{
            exptected_line.push(' ');
        }
        assert!(doc.lines == vec![exptected_line]);
        assert!(doc.cursors.last().unwrap().head.x() == TAB_WIDTH);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
    }

//DELETE
    #[test]
    fn single_cursor_delete_removes_char(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::delete_at_cursor(cursor, &mut doc.lines, &mut doc.modified);
        assert!(doc.lines == vec!["dk".to_string()]);
        assert!(cursor.head.x() == 0);
        assert!(cursor.head.y() == 0);
    }
    #[test]
    fn single_cursor_delete_at_end_of_line_appends_next_line_to_current(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::delete_at_cursor(cursor, &mut doc.lines, &mut doc.modified);
        assert!(doc.lines == vec!["idksomething".to_string()]);
        assert!(cursor.head.x() == 3);
        assert!(cursor.head.y() == 0);
    }
    #[test]
    fn single_cursor_delete_removes_selection(){
        assert!(false);
    }
    
//BACKSPACE
    #[test]
    fn single_cursor_backspace_removes_previous_character(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(1, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::backspace_at_cursor(cursor, &mut doc.lines, &mut doc.modified);
        println!("{:?}", doc.lines);
        assert!(doc.lines == vec!["dk".to_string()]);
        println!("{:?}", cursor.head);
        assert!(cursor.head.x() == 0);
        assert!(cursor.head.y() == 0);
    }
    #[test]
    fn single_cursor_backspace_at_start_of_line_appends_current_line_to_end_of_previous_line(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(0, 1);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::backspace_at_cursor(cursor, &mut doc.lines, &mut doc.modified);
        println!("{:?}", doc.lines);
        assert!(doc.lines == vec!["idksomething".to_string()]);
        println!("{:?}", cursor.head);
        assert!(cursor.head.x() == 3);
        assert!(cursor.head.y() == 0);
    }
    #[test]
    fn single_cursor_backspace_removes_previous_tab(){
        let mut doc = Document::default();
        let mut line = String::new();
        for _ in 0..TAB_WIDTH{
            line.push(' ');
        }
        line.push_str("something");
        doc.lines = vec![line];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(TAB_WIDTH, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::backspace_at_cursor(cursor, &mut doc.lines, &mut doc.modified);
        println!("{:?}", doc.lines);
        assert!(doc.lines == vec!["something".to_string()]);
        println!("{:?}", cursor.head);
        assert!(cursor.head.x() == 0);
        assert!(cursor.head.y() == 0);
    }
    
//MOVE CURSOR LEFT
    #[test]
    fn single_cursor_move_cursor_left_at_document_start_does_not_move_cursor(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::move_cursor_left(cursor, &doc.lines);
        assert!(cursor.head.y == 0);
        assert!(cursor.head.x == 0);
    }
    #[test]
    fn single_cursor_move_cursor_left_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(1, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y == 0);
        assert!(cursor.head.x == 1);
        Document::move_cursor_left(cursor, &doc.lines);
        assert!(cursor.head.y == 0);
        assert!(cursor.head.x == 0);
    }
    #[test]
    fn single_cursor_move_cursor_left_at_line_start_moves_cursor_to_previous_line_end(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(0, 1);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y == 1);
        assert!(cursor.head.x == 0);
        Document::move_cursor_left(cursor, &doc.lines);
        assert!(cursor.head.y == 0);
        assert!(cursor.head.x == 3);
    }
    
//MOVE CURSOR UP
    #[test]
    fn single_cursor_move_cursor_up_at_document_start_does_not_move_cursor(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::move_cursor_up(cursor, &doc.lines);
        assert!(cursor.head.y() == 0);
        assert!(cursor.head.x() == 0);
    }
    #[test]
    fn single_cursor_move_cursor_up_works_when_moving_to_shorter_line(){
        let mut doc = Document::default();
        doc.lines = vec!["1".to_string(), "123".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 1);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y() == 1);
        assert!(cursor.head.x() == 3);
        Document::move_cursor_up(cursor, &doc.lines);
        assert!(cursor.head.y() == 0);
        assert!(cursor.head.x() == 1);
    }
    #[test]
    fn single_cursor_move_cursor_up_works_when_moving_to_longer_line(){
        let mut doc = Document::default();
        doc.lines = vec!["1234".to_string(), "1".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(1, 1);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y() == 1);
        assert!(cursor.head.x() == 1);
        Document::move_cursor_up(cursor, &doc.lines);
        assert!(cursor.head.y() == 0);
        assert!(cursor.head.x() == 1);
    }

//MOVE CURSOR RIGHT
    #[test]
    fn single_cursor_move_cursor_right_at_document_end_does_not_move_cursor(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y() == 0);
        assert!(cursor.head.x() == 3);
        Document::move_cursor_right(cursor, &doc.lines);
        assert!(cursor.head.y() == 0);
        assert!(cursor.head.x() == 3);
    }
    #[test]
    fn single_cursor_move_cursor_right_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::move_cursor_right(cursor, &doc.lines);
        assert!(cursor.head.y() == 0);
        assert!(cursor.head.x() == 1);
    }
    #[test]
    fn single_cursor_move_cursor_right_at_line_end_moves_cursor_to_start_of_next_line(){
        let mut doc = Document::default();
        doc.lines = vec!["1".to_string(), "2".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(1, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y() == 0);
        assert!(cursor.head.x() == 1);
        Document::move_cursor_right(cursor, &doc.lines);
        assert!(cursor.head.y() == 1);
        assert!(cursor.head.x() == 0);
    }

//MOVE CURSOR DOWN
    #[test]
    fn single_cursor_move_cursor_down_at_document_end_does_not_move_cursor(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y() == 0);
        assert!(cursor.head.x() == 3);
        Document::move_cursor_down(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.y() == 0);
        assert!(cursor.head.x() == 3);
    }
    #[test]
    fn single_cursor_move_cursor_down_works_when_moving_to_shorter_line(){
        let mut doc = Document::default();
        doc.lines = vec!["123".to_string(), "1".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y() == 0);
        assert!(cursor.head.x() == 3);
        Document::move_cursor_down(cursor, &doc.lines);
        assert!(cursor.head.y() == 1);
        assert!(cursor.head.x() == 1);
    }
    #[test]
    fn single_cursor_move_cursor_down_works_when_moving_to_longer_line(){
        let mut doc = Document::default();
        doc.lines = vec!["1".to_string(), "1234".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(1, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.y() == 0);
        assert!(cursor.head.x() == 1);
        Document::move_cursor_down(cursor, &doc.lines);
        assert!(cursor.head.y() == 1);
        assert!(cursor.head.x() == 1);
    }

//move cursors page up
    #[test]
    fn single_cursor_move_cursor_page_up_works(){
        assert!(false);
    }
//move cursors page down
    #[test]
    fn single_cursor_move_cursor_page_down_works(){
        assert!(false);
    }

//MOVE CURSOR HOME
    #[test]
    fn single_cursor_move_cursor_home_moves_cursor_to_text_start_when_cursor_past_text_start(){
        let mut doc = Document::default();
        let mut line = String::new();
        for _ in 0..TAB_WIDTH{
            line.push(' ');
        }
        line.push_str("idk");
        doc.lines = vec![line];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(TAB_WIDTH + 2, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::move_cursor_home(cursor, &doc.lines);
        assert!(cursor.head.x == TAB_WIDTH);
        assert!(cursor.head.y == 0);
        assert!(cursor.anchor.x == TAB_WIDTH);
        assert!(cursor.anchor.y == 0);
    }
    #[test]
    fn single_cursor_move_cursor_home_moves_cursor_to_line_start_when_cursor_at_text_start(){
        let mut doc = Document::default();
        let mut line = String::new();
        for _ in 0..TAB_WIDTH{
            line.push(' ');
        }
        line.push_str("idk");
        doc.lines = vec![line];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(TAB_WIDTH, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.x == TAB_WIDTH);
        assert!(cursor.head.y == 0);
        assert!(cursor.anchor.x == TAB_WIDTH);
        assert!(cursor.anchor.y == 0);
        Document::move_cursor_home(cursor, &doc.lines);
        assert!(cursor.head.x == 0);
        assert!(cursor.head.y == 0);
        assert!(cursor.anchor.x == 0);
        assert!(cursor.anchor.y == 0);
    }
    #[test]
    fn single_cursor_move_cursor_home_moves_cursor_to_text_start_when_cursor_at_line_start(){
        let mut doc = Document::default();
        let mut line = String::new();
        for _ in 0..TAB_WIDTH{
            line.push(' ');
        }
        line.push_str("idk");
        doc.lines = vec![line];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::move_cursor_home(cursor, &doc.lines);
        assert!(cursor.head.x == TAB_WIDTH);
        assert!(cursor.head.y == 0);
        assert!(cursor.anchor.x == TAB_WIDTH);
        assert!(cursor.anchor.y == 0);
    }

//MOVE CURSOR END
    #[test]
    fn single_cursor_move_cursor_end_moves_cursor_to_line_end(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::move_cursor_end(cursor, &doc.lines);
        assert!(cursor.head.x == 3);
        assert!(cursor.head.y == 0);
        assert!(cursor.anchor.x == 3);
        assert!(cursor.anchor.y == 0);
    }

//MOVE CURSOR DOC START
    #[test]
    fn single_cursor_move_cursor_doc_start_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(9, 1);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.x == 9);
        assert!(cursor.head.y == 1);
        assert!(cursor.anchor.x == 9);
        assert!(cursor.anchor.y == 1);
        Document::move_cursor_document_start(cursor);
        assert!(cursor.head.x == 0);
        assert!(cursor.head.y == 0);
        assert!(cursor.anchor.x == 0);
        assert!(cursor.anchor.y == 0);
    }
//MOVE CURSOR DOC END
    #[test]
    fn single_cursor_move_cursor_doc_end_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::move_cursor_document_end(cursor, &doc.lines);
        assert!(cursor.head.x == 9);
        assert!(cursor.head.y == 1);
        assert!(cursor.anchor.x == 9);
        assert!(cursor.anchor.y == 1);
    }

//EXTEND SELECTION RIGHT
    #[test]
    fn single_cursor_extend_selection_right_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::extend_selection_right_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 1);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 0);
        assert!(cursor.anchor.y() == 0);
    }
    #[test]
    fn single_cursor_extend_selection_right_at_end_of_line_wraps_to_next_line(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_right_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 0);
        assert!(cursor.head.y() == 1);
        assert!(cursor.anchor.x() == 3);
        assert!(cursor.anchor.y() == 0);
    }
    #[test]
    fn single_cursor_extend_selection_right_at_document_end_does_not_extend_selection(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_right_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 3);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 3);
        assert!(cursor.anchor.y() == 0);
    }

//EXTEND SELECTION LEFT
    #[test]
    fn single_cursor_extend_selection_left_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_left_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 2);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 3);
        assert!(cursor.anchor.y() == 0);
    }
    #[test]
    fn single_cursor_extend_selection_left_at_start_of_line_wraps_to_previous_line(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(0, 1);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_left_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 9);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 0);
        assert!(cursor.anchor.y() == 1);
    }
    #[test]
    fn single_cursor_extend_selection_left_at_document_start_does_not_extend_selection(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::extend_selection_left_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 0);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 0);
        assert!(cursor.anchor.y() == 0);
    }

//EXTEND SELECTION UP
    #[test]
    fn single_cursor_extend_selection_up_works_when_previous_line_is_shorter(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(9, 1);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_up_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 3);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 9);
        assert!(cursor.anchor.y() == 1);
    }
    #[test]
    fn single_cursor_extend_selection_up_works_when_previous_line_is_longer(){
        let mut doc = Document::default();
        doc.lines = vec!["something".to_string(), "idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 1);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_up_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 3);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 3);
        assert!(cursor.anchor.y() == 1);
    }
    #[test]
    fn single_cursor_extend_selection_up_at_document_start_does_not_extend_selection(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::extend_selection_up_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 0);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 0);
        assert!(cursor.anchor.y() == 0);
    }
    
//EXTEND SELECTION DOWN
    #[test]
    fn single_cursor_extend_selection_down_works_when_next_line_shorter(){
        let mut doc = Document::default();
        doc.lines = vec!["something".to_string(), "idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(9, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_down_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 3);
        assert!(cursor.head.y() == 1);
        assert!(cursor.anchor.x() == 9);
        assert!(cursor.anchor.y() == 0);
    }
    #[test]
    fn single_cursor_extend_selection_down_works_when_next_line_longer(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_down_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 3);
        assert!(cursor.head.y() == 1);
        assert!(cursor.anchor.x() == 3);
        assert!(cursor.anchor.y() == 0);
    }
    #[test]
    fn single_cursor_extend_selection_down_at_document_end_does_not_extend_selection(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_down_at_cursor(cursor, &doc.lines);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 3);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 3);
        assert!(cursor.anchor.y() == 0);
    }
    
//EXTEND SELECTION HOME
    #[test]
    fn single_cursor_extend_selection_home_moves_cursor_head_to_text_start_when_cursor_past_text_start(){
        let mut doc = Document::default();
        let mut line = String::new();
        for _ in 0..TAB_WIDTH{
            line.push(' ');
        }
        line.push_str("idk");
        doc.lines = vec![line];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(TAB_WIDTH + 2, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_home_at_cursor(cursor, &doc.lines);
        assert!(cursor.head.x == TAB_WIDTH);
        assert!(cursor.head.y == 0);
        assert!(cursor.anchor.x == TAB_WIDTH + 2);
        assert!(cursor.anchor.y == 0);
    }
    #[test]
    fn single_cursor_extend_selection_home_moves_cursor_head_to_line_start_when_cursor_at_text_start(){
        let mut doc = Document::default();
        let mut line = String::new();
        for _ in 0..TAB_WIDTH{
            line.push(' ');
        }
        line.push_str("idk");
        doc.lines = vec![line];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(TAB_WIDTH, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        assert!(cursor.head.x == TAB_WIDTH);
        assert!(cursor.head.y == 0);
        assert!(cursor.anchor.x == TAB_WIDTH);
        assert!(cursor.anchor.y == 0);
        Document::extend_selection_home_at_cursor(cursor, &doc.lines);
        assert!(cursor.head.x == 0);
        assert!(cursor.head.y == 0);
        assert!(cursor.anchor.x == TAB_WIDTH);
        assert!(cursor.anchor.y == 0);
    }
    #[test]
    fn single_cursor_extend_selection_home_moves_cursor_head_to_text_start_when_cursor_at_line_start(){
        let mut doc = Document::default();
        let mut line = String::new();
        for _ in 0..TAB_WIDTH{
            line.push(' ');
        }
        line.push_str("idk");
        doc.lines = vec![line];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::extend_selection_home_at_cursor(cursor, &doc.lines);
        assert!(cursor.head.x == TAB_WIDTH);
        assert!(cursor.head.y == 0);
        assert!(cursor.anchor.x == 0);
        assert!(cursor.anchor.y == 0);
    }

//EXTEND SELECTION END
    #[test]
    fn single_cursor_extend_selection_end_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::extend_selection_end_at_cursor(cursor, &doc.lines);
        assert!(cursor.head.x() == 3);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 0);
        assert!(cursor.anchor.y() == 0);
    }

//extend selection page up
//extend selection page down
    
//COLLAPSE SELECTION CURSOR
    //when on same line and head less than anchor
    #[test]
    fn single_cursor_collapse_selection_cursors_works_when_on_same_line_and_head_less_than_anchor(){
        let mut doc = Document::default();
        doc.lines = vec!["something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(9, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_left_at_cursor(cursor, &doc.lines);
        Document::collapse_selection_cursor(cursor);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 8);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 8);
        assert!(cursor.anchor.y() == 0);
    }
    //when on same line and anchor less than head
    #[test]
    fn single_cursor_collapse_selection_cursors_works_when_on_same_line_and_anchor_less_than_head(){
        let mut doc = Document::default();
        doc.lines = vec!["something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        Document::extend_selection_right_at_cursor(cursor, &doc.lines);
        Document::collapse_selection_cursor(cursor);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 1);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 1);
        assert!(cursor.anchor.y() == 0);
    }
    //when on different lines and head less than anchor
    #[test]
    fn single_cursor_collapse_selection_cursors_works_when_on_different_lines_and_head_less_than_anchor(){
        let mut doc = Document::default();
        doc.lines = vec!["something".to_string(), "idk".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 1);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_up_at_cursor(cursor, &doc.lines);
        Document::collapse_selection_cursor(cursor);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 3);
        assert!(cursor.head.y() == 0);
        assert!(cursor.anchor.x() == 3);
        assert!(cursor.anchor.y() == 0);
    }
    //when on different lines and anchor less than head
    #[test]
    fn single_cursor_collapse_selection_cursors_works_when_on_different_lines_and_anchor_less_than_head(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        let cursor = doc.cursors.get_mut(0).unwrap();
        let position = Position::new(3, 0);
        Document::set_cursor_position(cursor, position, &doc.lines);
        Document::extend_selection_down_at_cursor(cursor, &doc.lines);
        Document::collapse_selection_cursor(cursor);
        println!("{:?}", cursor);
        assert!(cursor.head.x() == 3);
        assert!(cursor.head.y() == 1);
        assert!(cursor.anchor.x() == 3);
        assert!(cursor.anchor.y() == 1);
    }

    //clamp cursors to line end (verified in cursor movement tests)
    //clamp selection cursors to line end (verified in extend selection tests)

    //goto

    #[test]
    fn len_returns_last_line_number(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "some".to_string(), "shit".to_string()];
        assert!(doc.lines().len() == 3);
    }

    //scroll client view down
    //scroll client view left
    //scroll client view right
    //scroll client view up
    //scroll view following cursor

    //set client view size (does this need testing?)
    //get client view text
    //get client view line numbers
    //get client cursor positions
//}
