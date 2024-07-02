use crate::Position;
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
    // should a view offset be stored here instead of the client?
    // this would allow us to return a viewable buffer whether the view is
    // clamped to cursor position, or if view ignores cursor position
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
        self.cursor_head
    }

    pub fn _cursor_head(&self) -> Position{
        self.cursor_head
    }

    pub fn _cursor_anchor(&self) -> Position{
        self.cursor_anchor
    }

    //for doc tests
    // ensure vertical_start is always less than vertical_end
    // ensure horizontal_start is always less than horizontal_end
    // if vertical_end - vertical_start is greater than document length, just return the available lines
    // if horizontal_end - horizontal_start is greater than line length, just return the available chars
    pub fn lines_in_bounds(&self, vertical_start: usize, vertical_end: usize, horizontal_start: usize, horizontal_end: usize) -> Vec<String>{
        // get all lines from vertical_start to vertical_end
        // get all chars from horizontal_start to horizontal_end
        // create new Vec
        // create new String
        // push all chars to string
        // push all lines to vec
        // return buffer
        let mut new_lines = String::new();
        for y in vertical_start..=vertical_end{
            let line = &self.lines[y];
            let bounded_line = line.get(horizontal_start..=horizontal_end).unwrap();
            new_lines.push_str(bounded_line);
        }
        vec![String::new()]
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

    pub fn insert_char(&mut self, c: char){self.modified = true;
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

    pub fn move_cursor_page_up(&mut self, terminal_height: usize){
        (self.cursor_anchor.y, self.cursor_head.y) = if self.cursor_position().y >= terminal_height{
            // pages up while still displaying first line from previous page
            (
                self.cursor_position().y.saturating_sub(terminal_height - 1),
                self.cursor_position().y.saturating_sub(terminal_height - 1)
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

    pub fn move_cursor_page_down(&mut self, terminal_height: usize){
        let document_length = self.len();
        (self.cursor_anchor.y, self.cursor_head.y) = if self.cursor_position().y.saturating_add(terminal_height) <= document_length{
            // pages down while still displaying last line from previous page
            (
                self.cursor_position().y.saturating_add(terminal_height - 1),
                self.cursor_position().y.saturating_add(terminal_height - 1)
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
            self.cursor_head.y = line_number;
            self.cursor_anchor.y = line_number;
            self.clamp_cursor_to_line_end();
            Ok(())
        }
        else{
            Err(())
        }
    }
}





#[cfg(test)]
mod tests{
    use crate::document::Document;
    
    #[test]
    fn move_left_cannot_go_before_doc_start(){
        let mut doc = Document::default();
        doc.move_cursor_left();
        assert!(doc.cursor_position().x() == 0);
        assert!(doc._cursor_head().x() == 0);
        assert!(doc._cursor_anchor().x() == 0);
    }

    #[test]
    fn move_up_cannot_go_above_doc_start(){
        let mut doc = Document::default();
        doc.move_cursor_up();
        assert!(doc.cursor_position().y() == 0);
        assert!(doc._cursor_head().y() == 0);
        assert!(doc._cursor_anchor().y() == 0);
    }

    #[test]
    fn move_right_cannot_go_beyond_doc_end(){
        let mut doc = Document::default();
        doc.move_cursor_right();
        assert!(doc.cursor_position().x() == 0);
        assert!(doc._cursor_head().x() == 0);
        assert!(doc._cursor_anchor().x() == 0);
    }

    #[test]
    fn move_down_cannot_go_below_doc_end(){
        let mut doc = Document::default();
        doc.move_cursor_down();
        assert!(doc.cursor_position().y() == 0);
        assert!(doc._cursor_head().y() == 0);
        assert!(doc._cursor_anchor().y() == 0);
    }
}
