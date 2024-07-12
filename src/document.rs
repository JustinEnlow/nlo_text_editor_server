use crate::{Position, View};
use std::fs;
use std::io::{Error, Write};
use unicode_segmentation::UnicodeSegmentation;

// tab keypress inserts the number of spaces specified in TAB_WIDTH into the focused document
pub const TAB_WIDTH: usize = 4;



pub struct Document{
    lines: Vec<String>,
    file_name: Option<String>,
    modified: bool,
    cursor_anchor: Position,
    cursor_head: Position,
    stored_line_position: usize,
    client_view: View,
}
impl Default for Document{
    fn default() -> Self {
        Self{
            lines: vec![String::new()],
            file_name: None,
            modified: false,
            cursor_anchor: Position::default(),
            cursor_head: Position::default(),
            stored_line_position: 0,
            client_view: View::default(),
        }
    }
}
impl Document{
    // not sure whether to continue using &str or &Path
    // everything seems to be working fine so far with just &str
    pub fn open(path: &str) -> Result<Self, std::io::Error>{
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
            file_name: Some(path.to_string()),
            modified: false,
            cursor_anchor: Position::default(),
            cursor_head: Position::default(),
            stored_line_position: 0,
            client_view: View::default(),
        })
    }

    pub fn set_client_view_size(&mut self, width: usize, height: usize){
        self.client_view.width = width;
        self.client_view.height = height;
    }
    //for doc tests
    // ensure vertical_start is always less than vertical_end
    // ensure horizontal_start is always less than horizontal_end
    // if vertical_end - vertical_start is greater than document length, just return the available lines
    // if horizontal_end - horizontal_start is greater than line length, just return the available chars
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
                client_view_line_numbers.push_str(&format!("{}\n", (y+1).to_string()))
            }
        }

        client_view_line_numbers
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
        self.cursor_head
    
    }
    fn set_cursor_position(&mut self, position: Position){
        if let Some(line) = self.lines.get(position.y()){
            if position.x() <= line.graphemes(true).count(){
                self.cursor_anchor = position;
                self.cursor_head = position;
                self.stored_line_position = position.x();
            }else{
                let new_pos = Position::new(line.graphemes(true).count(), position.y());
                self.cursor_anchor = new_pos;
                self.cursor_head = new_pos;
                self.stored_line_position = new_pos.x();
            }
        }
    }

    pub fn _cursor_head(&self) -> Position{
        self.cursor_head
    }

    pub fn _cursor_anchor(&self) -> Position{
        self.cursor_anchor
    }

    //TODO: verify functionality
    pub fn get_client_cursor_position(&self) -> Option<Position>{
        //only get a cursor position, if the cursor is within view
        if self.cursor_position().x() >= self.client_view.horizontal_start 
        && self.cursor_position().x() < (self.client_view.horizontal_start + self.client_view.width){
            if self.cursor_position().y() >= self.client_view.vertical_start
            && self.cursor_position().y() < (self.client_view.vertical_start + self.client_view.height){
                return Some(
                    Position{
                        x: self.cursor_position().x() - self.client_view.horizontal_start,
                        y: self.cursor_position().y() - self.client_view.vertical_start
                    }
                );
            }
        }

        None
    }

    pub fn lines_as_single_string(&self) -> String{
        let mut lines = String::new();
        for idk in self.lines.clone(){
            lines.push_str(format!("{}\n", idk).as_str())
        }
        lines
    }

    pub fn current_line(&self) -> &String{
        match self.lines.get(self.cursor_position().y){
            Some(line) => line,
            None => panic!("No line at cursor position. This should be impossible")
        }
    }
    pub fn current_line_mut(&mut self) -> &mut String{
        let current_line_index = self.cursor_position().y;
        match self.lines.get_mut(current_line_index){
            Some(line) => line,
            None => panic!("No line at cursor position. This should be impossible")
        }
    }

    pub fn is_modified(&self) -> bool{
        self.modified
    }

    // returns the number of lines in this document
    pub fn len(&self) -> usize{
        self.lines.len()
    }

    pub fn get_first_non_whitespace_character_index(&self)-> usize{
        if self.current_line().is_empty(){
            return 0;
        }
        for (index, grapheme) in self.current_line()[..].graphemes(true).enumerate(){
            if grapheme == " "{/*do nothing*/}
            else{
                return index;
            }
        }

        0
    }

    pub fn slice_is_all_spaces(&self, start_of_slice: usize, end_of_slice: usize) -> bool{
        for grapheme in self.current_line()[start_of_slice..end_of_slice].graphemes(true){
            if grapheme != " "{
                return false;
            }
        }

        true
    }

    fn distance_to_next_multiple_of_tab_width(&self) -> usize{
        if self.cursor_position().x % TAB_WIDTH != 0{
            TAB_WIDTH - (self.cursor_position().x % TAB_WIDTH)
        }else{
            0
        }
    }

    //fn get_char_at_cursor(&mut self) -> Option<char>{
    //    let line = self.current_line();
    //
    //    let mut char_at_cursor = Some(' ');
    //    for (index, grapheme) in line.graphemes(true).enumerate(){
    //        if index == self.cursor_position().x{
    //            char_at_cursor = grapheme.chars().next();
    //        }
    //    }
    //
    //    char_at_cursor
    //}

    fn cursor_on_last_line(&self) -> bool{
        self.cursor_position().y.saturating_add(1) == self.len()
    }

    fn cursor_at_end_of_line(&self) -> bool{
        self.cursor_position().x == self.current_line().graphemes(true).count()
    }

    fn set_current_line(&mut self, new_line: String){
        *self.current_line_mut() = new_line;
    }

    pub fn insert_newline(&mut self){
        self.modified = true;
        
        let line = self.current_line();
        
        let mut modified_current_line: String = String::new();
        let mut new_line: String = String::new();
        for (index, grapheme) in line[..].graphemes(true).enumerate(){
            if index < self.cursor_position().x{
                modified_current_line.push_str(grapheme);
            }
            else{
                new_line.push_str(grapheme);
            }
        }
            
        self.set_current_line(modified_current_line);
        self.lines.insert(self.cursor_position().y.saturating_add(1), new_line);
        self.move_cursor_right();
    }

    pub fn insert_char(&mut self, c: char){
        self.modified = true;
        
        let horizontal_index = self.cursor_position().x;
        
        let line = self.current_line_mut();
        line.insert(horizontal_index, c);
        self.move_cursor_right();
    }

    pub fn delete(&mut self){
        match (self.cursor_on_last_line(), self.cursor_at_end_of_line()){
            (true, true) => {/*do nothing*/}
            (_, false) => self.delete_next_char(),
            (false, true) => self.delete_next_newline(),
        }
    }

    fn delete_next_char(&mut self){
        self.modified = true;

        let line = self.current_line();
        let mut result = String::new();
        for (index, grapheme) in line[..].graphemes(true).enumerate(){
            if index != self.cursor_position().x{
                result.push_str(grapheme);
            }
        }
        self.set_current_line(result);
    }

    
    fn delete_next_newline(&mut self){
        self.modified = true;

        let next_line = self.lines.remove(self.cursor_position().y.saturating_add(1));
        let line = self.current_line();
        self.set_current_line(format!("{}{}", line, next_line));
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

    pub fn tab(&mut self){
        let tab_distance = self.distance_to_next_multiple_of_tab_width();
        let modified_tab_width = if tab_distance > 0 && tab_distance < TAB_WIDTH{
            tab_distance
        }else{
            TAB_WIDTH
        };
        for _ in 0..modified_tab_width{
            self.insert_char(' ');
        }
    }

    // auto_indent not working well with undo/redo
    pub fn enter(&mut self){
        // auto indent doesn't work correctly if previous line has only whitespace characters
        // also doesn't auto indent for first line of function bodies, because function declaration
        // is at lower indentation level
        let indent_level = self.get_first_non_whitespace_character_index();
        self.insert_newline();
        // auto indent
        if indent_level != 0{
            for _ in 0..indent_level{
                self.insert_char(' ');
            }
        }
    }

    pub fn backspace(&mut self){
        if self.cursor_position().x >= TAB_WIDTH
        // handles case where user adds a space after a tab, and wants to delete only the space
        && self.cursor_position().x % TAB_WIDTH == 0
        // if previous 4 chars are spaces, delete 4. otherwise, use default behavior
        && self.slice_is_all_spaces(
            self.cursor_position().x - TAB_WIDTH, 
            self.cursor_position().x
        ){
            self.delete_prev_tab();
        }
        else if self.cursor_position().x > 0{
            self.delete_prev_char();
        }
        else if self.cursor_position().x == 0 && self.cursor_position().y > 0{
            self.delete_prev_newline();
        }
    }

    fn delete_prev_char(&mut self){
        self.move_cursor_left();
        self.delete_next_char();
    }

    fn delete_prev_newline(&mut self){
        while self.cursor_position().x > 0{
            self.delete_prev_char();
        }
        self.move_cursor_left();
        self.delete_next_newline();
    }

    fn delete_prev_tab(&mut self){
        for _ in 0..TAB_WIDTH{
            self.move_cursor_left();
            self.delete_next_char();
        }
    }

    // would checking returning client view changed bool here allow us to reduce DisplayView server response calls?
    pub fn scroll_view_following_cursor(&mut self) -> bool{
        let mut should_update_client_view = false;

        if self.cursor_position().y() < self.client_view.vertical_start{
            self.client_view.vertical_start = self.cursor_position().y();
            should_update_client_view = true;
        }
        else if self.cursor_position().y() >= self.client_view.vertical_start.saturating_add(self.client_view.height){
            self.client_view.vertical_start = self.cursor_position().y().saturating_sub(self.client_view.height).saturating_add(1);
            should_update_client_view = true;
        }
    
        if self.cursor_position().x() < self.client_view.horizontal_start{
            self.client_view.horizontal_start = self.cursor_position().x();
            should_update_client_view = true;
        }
        else if self.cursor_position().x() >= self.client_view.horizontal_start.saturating_add(self.client_view.width){
            self.client_view.horizontal_start = self.cursor_position().x().saturating_sub(self.client_view.width).saturating_add(1);
            should_update_client_view = true;
        }

        should_update_client_view
    }

    pub fn move_cursor_up(&mut self){
        self.cursor_anchor.y = self.cursor_position().y.saturating_sub(1);
        self.cursor_head.y = self.cursor_position().y.saturating_sub(1);
        self.clamp_cursor_to_line_end();
    }

    pub fn move_cursor_down(&mut self){
        if self.cursor_position().y.saturating_add(1) < self.len(){
            self.cursor_anchor.y = self.cursor_position().y.saturating_add(1);
            self.cursor_head.y = self.cursor_position().y.saturating_add(1);
        }
        self.clamp_cursor_to_line_end();
    }

    pub fn move_cursor_right(&mut self){
        let line_width = self.current_line().graphemes(true).count();
        if self.cursor_position().x < line_width{
            self.cursor_anchor.x = self.cursor_position().x.saturating_add(1);
            self.cursor_head.x = self.cursor_position().x.saturating_add(1);
        }
        else if self.cursor_position().y < self.len().saturating_sub(1){
            self.cursor_anchor.y = self.cursor_anchor.y.saturating_add(1);
            self.cursor_head.y = self.cursor_head.y.saturating_add(1);

            self.cursor_anchor.x = 0;
            self.cursor_head.x = 0;
        }
        self.stored_line_position = self.cursor_position().x;
    }

    pub fn move_cursor_left(&mut self){
        if self.cursor_position().x > 0{
            self.cursor_anchor.x = self.cursor_anchor.x.saturating_sub(1);
            self.cursor_head.x = self.cursor_head.x.saturating_sub(1);
        }
        else if self.cursor_position().y > 0{
            self.cursor_anchor.y = self.cursor_anchor.y.saturating_sub(1);
            self.cursor_head.y = self.cursor_head.y.saturating_sub(1);

            self.cursor_anchor.x = self.current_line().graphemes(true).count();
            self.cursor_head.x = self.current_line().graphemes(true).count();
        }
        self.stored_line_position = self.cursor_position().x;
    }

    pub fn move_cursor_page_up(&mut self/*, terminal_height: usize*/){
        (self.cursor_anchor.y, self.cursor_head.y) = if self.cursor_position().y >= self.client_view.height/*terminal_height*/{
            // pages up while still displaying first line from previous page
            (
                self.cursor_position().y.saturating_sub(self.client_view.height/*terminal_height*/ - 1),
                self.cursor_position().y.saturating_sub(self.client_view.height/*terminal_height*/ - 1)
            )
            // to disregard first line on previous page, and do full page up use:
            /*(
                self.cursor_position().y.saturating_sub(terminal_height),
                self.cursor_position().y.saturating_sub(terminal_height)
            )*/
        }else{
            (0, 0)
        };
        self.clamp_cursor_to_line_end();
    }

    pub fn move_cursor_page_down(&mut self/*, terminal_height: usize*/){
        let document_length = self.len();
        (self.cursor_anchor.y, self.cursor_head.y) = if self.cursor_position().y.saturating_add(self.client_view.height/*terminal_height*/) <= document_length{
            // pages down while still displaying last line from previous page
            (
                self.cursor_position().y.saturating_add(self.client_view.height/*terminal_height*/ - 1),
                self.cursor_position().y.saturating_add(self.client_view.height/*terminal_height*/ - 1)
            )
            // to disregard last line on previous page, and do full page down use:
            /*(
                self.cursor_position().y.saturating_add(terminal_height),
                self.cursor_position().y.saturating_add(terminal_height)
            )*/
        }else{
            (
                document_length.saturating_sub(1),
                document_length.saturating_sub(1)
            )
        };
        self.clamp_cursor_to_line_end();
    }

    pub fn move_cursor_home(&mut self){
        let start_of_line = self.get_first_non_whitespace_character_index();
        if self.cursor_position().x == start_of_line{
            self.cursor_anchor.x = 0;
            self.cursor_head.x = 0;
        }else{
            self.cursor_anchor.x = start_of_line;
            self.cursor_head.x = start_of_line;
        }
        self.stored_line_position = self.cursor_position().x;
    }

    pub fn move_cursor_end(&mut self){
        let line_width = self.current_line().graphemes(true).count();
        self.cursor_anchor.x = line_width;
        self.cursor_head.x = line_width;
        self.stored_line_position = self.cursor_position().x;
    }

    pub fn move_cursor_document_start(&mut self){
        self.cursor_anchor.x = 0;
        self.cursor_anchor.y = 0;
        self.cursor_head.x = 0;
        self.cursor_head.y = 0;
        self.stored_line_position = self.cursor_position().x;
    }

    pub fn move_cursor_document_end(&mut self){
        self.cursor_anchor.y = self.len().saturating_sub(1);
        self.cursor_head.y = self.len().saturating_sub(1);
        
        let line_width = self.current_line().graphemes(true).count();

        self.cursor_anchor.x = line_width;
        self.cursor_head.x = line_width;
        self.stored_line_position = self.cursor_position().x;
    }

    pub fn scroll_client_view_down(&mut self, amount: usize){
        if self.client_view.vertical_start + self.client_view.height + amount <= self.len(){
            self.client_view.vertical_start = self.client_view.vertical_start + amount;
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
            self.client_view.horizontal_start = self.client_view.horizontal_start + amount;
        }
    }
    pub fn scroll_client_view_up(&mut self, amount: usize){
        self.client_view.vertical_start = self.client_view.vertical_start.saturating_sub(amount);
    }

    pub fn collapse_selection_cursors(&mut self){
        self.cursor_head.x = self.stored_line_position;
    }

    pub fn extend_selection_right(&mut self){
        let line_width = self.current_line().graphemes(true).count();
        if self.cursor_head.x < line_width{
            self.cursor_head.x = self.cursor_position().x.saturating_add(1);
        }
        else if self.cursor_head.y < self.len().saturating_sub(1){
            self.cursor_head.y = self.cursor_head.y.saturating_add(1);
            self.cursor_head.x = 0;
        }
        self.stored_line_position = self.cursor_head.x;
    }

    pub fn extend_selection_left(&mut self){
        if self.cursor_head.x > 0{
            self.cursor_head.x = self.cursor_head.x.saturating_sub(1);
        }
        else if self.cursor_head.y > 0{
            self.cursor_head.y = self.cursor_head.y.saturating_sub(1);
            self.cursor_head.x = self.current_line().graphemes(true).count();
        }
        self.stored_line_position = self.cursor_head.x;
    }

    pub fn extend_selection_up(&mut self){
        self.cursor_head.y = self.cursor_head.y.saturating_sub(1);
        self.clamp_selection_cursor_to_line_end();
    }

    pub fn extend_selection_down(&mut self){
        if self.cursor_head.y < self.len().saturating_sub(1){
            self.cursor_head.y = self.cursor_head.y.saturating_add(1);
        }
        self.clamp_selection_cursor_to_line_end();
    }

    pub fn extend_selection_home(&mut self){
        let start_of_line = self.get_first_non_whitespace_character_index();
        if self.cursor_head.x == start_of_line{
            self.cursor_head.x = 0;
        }else{
            self.cursor_head.x = start_of_line;
        }
        self.stored_line_position = self.cursor_head.x;
    }

    pub fn extend_selection_end(&mut self){
        let line_width = self.current_line().graphemes(true).count();
        self.cursor_head.x = line_width;
        self.stored_line_position = self.cursor_head.x;
    }

    pub fn _extend_selection_page_up(&mut self){}

    pub fn _extend_selection_page_down(&mut self){}

    fn clamp_cursor_to_line_end(&mut self){
        let line_width = self.current_line().graphemes(true).count();
        
        (self.cursor_anchor.x, self.cursor_head.x) = if self.cursor_position().x > line_width
                                                    || self.stored_line_position > line_width
        {
            (line_width, line_width)
        }else{
            (self.stored_line_position, self.stored_line_position)
        };
    }

    fn clamp_selection_cursor_to_line_end(&mut self){
        let line_width = self.current_line().graphemes(true).count();
        
        self.cursor_head.x = if self.cursor_head.x > line_width 
                            || self.stored_line_position > line_width
        {
            line_width
        }else{
            self.stored_line_position
        };
    }

    pub fn go_to(&mut self, line_number: usize) -> Result<(), ()>{
        if line_number < self.len(){
            //self.cursor_head.y = line_number;
            //self.cursor_anchor.y = line_number;
            //self.clamp_cursor_to_line_end();
            self.set_cursor_position(Position::new(self.stored_line_position, line_number));
            Ok(())
        }
        else{
            Err(())
        }
    }
}





//#[cfg(test)]
//mod tests{
//    use crate::{document::Document, Position};
    
    #[test]
    fn verify_set_cursor_position_behavior(){
        let mut doc = Document::default();
        doc.lines = vec!["1234".to_string(), "1".to_string(), "123".to_string()];

        // setting y inside doc should work
        doc.set_cursor_position(Position::new(0, 2));
        assert!(doc.cursor_position().y() == 2);
        assert!(doc.cursor_position().x() == 0);
        // setting y past doc end does nothing
        doc.set_cursor_position(Position::new(0, 3));
        assert!(doc.cursor_position().y() == 2);
        assert!(doc.cursor_position().x() == 0);
        // setting x past line end should restrict x to line end
        doc.set_cursor_position(Position::new(4, 2));
        assert!(doc.cursor_position().y() == 2);
        assert!(doc.cursor_position().x() == 3);
    }

    #[test]
    fn verify_move_cursor_left_behavior(){
        let mut doc = Document::default();
        doc.lines = vec!["123".to_string(), "123".to_string()];
        
        let position = Position::new(0, 1);
        doc.set_cursor_position(position);
        assert!(doc.cursor_position().y() == 1);
        assert!(doc.cursor_position().x() == 0);
        // if at line start, moves cursor to previous line end
        doc.move_cursor_left();
        assert!(doc.cursor_position().y() == 0);
        assert!(doc.cursor_position().x() == 3);
        // moves cursor left one char within same line
        doc.move_cursor_left();
        doc.move_cursor_left();
        doc.move_cursor_left();
        assert!(doc.cursor_position().y() == 0);
        assert!(doc.cursor_position().x() == 0);
        doc.move_cursor_left();
        // if at document start, does not move cursor
        assert!(doc.cursor_position().y() == 0);
        assert!(doc.cursor_position().x() == 0);
    }
    #[test]
    fn verify_move_cursor_up_behavior(){
        let mut doc = Document::default();
        doc.lines = vec!["1234".to_string(), "1".to_string(), "123".to_string()];

        let position = Position::new(3, 2);
        doc.set_cursor_position(position);
        assert!(doc.cursor_position().y() == 2);
        assert!(doc.cursor_position().x() == 3);
        // cursor moves up one line, if this line is shorter, cursor moves to line end
        doc.move_cursor_up();
        assert!(doc.cursor_position().y() == 1);
        assert!(doc.cursor_position().x() == 1);
        // cursor moves up one line, if this line is longer, cursor goes back to stored line position
        doc.move_cursor_up();
        assert!(doc.cursor_position().y() == 0);
        assert!(doc.cursor_position().x() == 3);
        // if at top line, does not move cursor
        doc.move_cursor_up();
        assert!(doc.cursor_position().y() == 0);
        assert!(doc.cursor_position().x() == 3);
    }

    #[test]
    fn verify_move_cursor_right_behavior(){
        let mut doc = Document::default();
        doc.lines = vec!["1".to_string(), "1".to_string()];

        assert!(doc.cursor_position().y() == 0);
        assert!(doc.cursor_position().x() == 0);
        // move cursor right one char within same line
        doc.move_cursor_right();
        assert!(doc.cursor_position().y() == 0);
        assert!(doc.cursor_position().x() == 1);
        // if at line end, move cursor to next line start
        doc.move_cursor_right();
        assert!(doc.cursor_position().y() == 1);
        assert!(doc.cursor_position().x() == 0);
        // move cursor right one char within same line
        doc.move_cursor_right();
        assert!(doc.cursor_position().y() == 1);
        assert!(doc.cursor_position().x() == 1);
        // if at doc end, does not move cursor
        doc.move_cursor_right();
        assert!(doc.cursor_position().y() == 1);
        assert!(doc.cursor_position().x() == 1);
    }

    #[test]
    fn verify_move_cursor_down_behavior(){
        let mut doc = Document::default();
        doc.lines = vec!["123".to_string(), "1".to_string(), "1234".to_string()];

        let position = Position::new(3, 0);
        doc.set_cursor_position(position);
        assert!(doc.cursor_position().y() == 0);
        assert!(doc.cursor_position().x() == 3);
        // cursor moves down one line, if this line is shorter, cursor moves to line end
        doc.move_cursor_down();
        assert!(doc.cursor_position().y() == 1);
        assert!(doc.cursor_position().x() == 1);
        // cursor moves down one line, if this line is longer, cursor goes back to stored line position
        doc.move_cursor_down();
        assert!(doc.cursor_position().y() == 2);
        assert!(doc.cursor_position().x() == 3);
        // if at bottom line, does not move cursor
        doc.move_cursor_down();
        assert!(doc.cursor_position().y() == 2);
        assert!(doc.cursor_position().x() == 3);
    }

    #[test]
    fn len_returns_last_line_number(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "some".to_string(), "shit".to_string()];
        assert!(doc.len() == 3);
    }
//}
