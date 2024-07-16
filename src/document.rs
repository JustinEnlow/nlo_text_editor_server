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
    fn set_cursor_position(&mut self, position: Position){
        if self.cursors.len() > 1{return;}

        let cursor = self.cursors.last_mut().unwrap();
        if let Some(line) = self.lines.get(position.y()){
            if position.x() <= line.graphemes(true).count(){
                cursor.anchor = position;
                cursor.head = position;
                cursor.stored_line_position = position.x();
            }else{
                let new_pos = Position::new(line.graphemes(true).count(), position.y());
                cursor.anchor = new_pos;
                cursor.head = new_pos;
                cursor.stored_line_position = new_pos.x();
            }
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

    // auto_indent not working well with undo/redo
    pub fn enter(&mut self){
        self.modified = true;
        // auto indent doesn't work correctly if previous line has only whitespace characters
        // also doesn't auto indent for first line of function bodies, because function declaration
        // is at lower indentation level
        for cursor in self.cursors.iter_mut(){
            let line = match self.lines.get(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            //let start_of_line = get_first_non_whitespace_character_index(line);
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
        
            let line_at_cursor = match self.lines.get_mut(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            *line_at_cursor = modified_current_line;
            self.lines.insert(cursor.head.y.saturating_add(1), new_line);
            // auto indent
            //if start_of_line != 0{
            //    let line = match self.lines.get_mut(cursor.head.y){
            //        Some(line) => line,
            //        None => panic!("No line at cursor position. This should be impossible")
            //    };
            //    for _ in 0..start_of_line{
            //        //self.insert_char(' ');
            //        Document::insert_char(' ', cursor, line, document_length, &mut self.stored_line_position, &mut self.modified);
            //    }
            //}
        }
        self.move_cursors_right();
    }

    pub fn insert_char_at_cursors(&mut self, c: char){
        for cursor in self.cursors.iter_mut(){
            Document::insert_char(c, cursor, &mut self.lines, &mut self.modified);
        }
    }
    fn insert_char(c: char, cursor: &mut Cursor, lines: &mut Vec<String>, modified: &mut bool){
        let line = match lines.get_mut(cursor.head.y){
            Some(line) => line,
            None => panic!("No line at cursor position. This should be impossible")
        };
        *modified = true;
        line.insert(cursor.head.x, c);
        Document::move_cursor_right(cursor, &lines);
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
                Document::insert_char(' ', cursor, &mut self.lines, &mut self.modified);
            }
        }
    }

    pub fn delete(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::delete_at_cursor(cursor, &mut self.lines, &mut self.modified);
        }
    }
    fn delete_at_cursor(cursor: &mut Cursor, lines: &mut Vec<String>, modified: &mut bool){
        let document_length = lines.len();
        *modified = true;
        if cursor.head.x == cursor.anchor.x && cursor.head.y == cursor.anchor.y{
            let line = match lines.get(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            let cursor_on_last_line = cursor.head.y.saturating_add(1) == document_length;
            let cursor_at_end_of_line = cursor.head.x == line.graphemes(true).count();
            match (cursor_on_last_line, cursor_at_end_of_line){
                (true, true) => {/*do nothing*/}
                (_, false) => {
                    let line = match lines.get_mut(cursor.head.y){
                        Some(line) => line,
                        None => panic!("No line at cursor position. This should be impossible")
                    };
                    if cursor.head.x < line.graphemes(true).count(){
                        line.remove(cursor.head.x);
                    }
                }
                (false, true) => {
                    let next_line = lines.remove(cursor.head.y.saturating_add(1));
                    let line = match lines.get_mut(cursor.head.y){
                        Some(line) => line,
                        None => panic!("No line at cursor position. This should be impossible")
                    };
                    line.push_str(&next_line);
                }
            }
        }else{
            // delete selection
        }
    }

    pub fn backspace(&mut self){
        for cursor in self.cursors.iter_mut(){
            if cursor.head.x >= TAB_WIDTH
            // handles case where user adds a space after a tab, and wants to delete only the space
            && cursor.head.x % TAB_WIDTH == 0
            // if previous 4 chars are spaces, delete 4. otherwise, use default behavior
            && slice_is_all_spaces(
                match self.lines.get(cursor.head.y){
                    Some(line) => line,
                    None => panic!("No line at cursor position. This should be impossible")
                },
                cursor.head.x - TAB_WIDTH, 
                cursor.head.x,
            ){
                for _ in 0..TAB_WIDTH{
                    Document::move_cursor_left(cursor, &self.lines);
                    Document::delete_at_cursor(cursor, &mut self.lines, &mut self.modified);
                }
            }
            else if cursor.head.x > 0{
                Document::move_cursor_left(cursor, &self.lines);
                Document::delete_at_cursor(cursor, &mut self.lines, &mut self.modified);
            }
            else if cursor.head.x == 0 && cursor.head.y > 0{
                Document::move_cursor_left(cursor, &self.lines);
                Document::delete_at_cursor(cursor, &mut self.lines, &mut self.modified);
            }
        }
    }

    pub fn move_cursors_up(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::move_cursor_up(cursor);
            Document::clamp_cursor_to_line_end(cursor, &self.lines);
        }
    }
    fn move_cursor_up(cursor: &mut Cursor){
        cursor.set_both_y(cursor.head.y.saturating_sub(1));
    }

    pub fn move_cursors_down(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::move_cursor_down(cursor, &self.lines);
            Document::clamp_cursor_to_line_end(cursor, &self.lines);
        }
    }
    fn move_cursor_down(cursor: &mut Cursor, lines: &Vec<String>){
        let document_length = lines.len();
        if cursor.head.y.saturating_add(1) < document_length{
            cursor.set_both_y(cursor.head.y.saturating_add(1));
        }
    }

    pub fn move_cursors_right(&mut self){
        for cursor in self.cursors.iter_mut(){
            Document::move_cursor_right(cursor, &self.lines);
        }
    }
    fn move_cursor_right(cursor: &mut Cursor, lines: &Vec<String>){
        let document_length = lines.len();
        let line = match lines.get(cursor.head.y){
            Some(line) => line,
            None => panic!("No line at cursor position. This should be impossible")
        };
        let line_width = line.graphemes(true).count();

        if cursor.head.x < line_width{
            cursor.set_both_x(cursor.head.x.saturating_add(1));
        }
        //move cursor to next line start
        else if cursor.head.y < document_length.saturating_sub(1){
            cursor.anchor.y = cursor.anchor.y.saturating_add(1);
            cursor.head.y = cursor.head.y.saturating_add(1);

            cursor.set_both_x(0);
        }
        cursor.stored_line_position = cursor.head.x;
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

            let line = match lines.get(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            cursor.set_both_x(line.graphemes(true).count());
        }
        cursor.stored_line_position = cursor.head.x;
    }

    pub fn move_cursors_page_up(&mut self){
        for cursor in self.cursors.iter_mut(){
            cursor.set_both_y(
                if cursor.head.y >= self.client_view.height{
                    cursor.head.y.saturating_sub(self.client_view.height - 1)
                }else{0}
            );
            Document::clamp_cursor_to_line_end(cursor, &self.lines);
        }
    }

    pub fn move_cursor_page_down(&mut self){
        let document_length = self.len();
        for cursor in self.cursors.iter_mut(){
            cursor.set_both_y(
                if cursor.head.y.saturating_add(self.client_view.height) <= document_length{
                    cursor.head.y.saturating_add(self.client_view.height - 1)
                }else{
                    document_length.saturating_sub(1)
                }
            );
            Document::clamp_cursor_to_line_end(cursor, &self.lines);
        }
    }

    pub fn move_cursors_home(&mut self){
        for cursor in self.cursors.iter_mut(){
            let line = match self.lines.get(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            let start_of_line = get_first_non_whitespace_character_index(line);
            if cursor.head.x == start_of_line{
                cursor.set_both_x(0);
            }else{
                cursor.set_both_x(start_of_line);
            }
            cursor.stored_line_position = cursor.head.x;
        }
    }

    pub fn move_cursors_end(&mut self){
        for cursor in self.cursors.iter_mut(){
            let line = match self.lines.get(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            let line_width = line.graphemes(true).count();
            cursor.set_both_x(line_width);
            cursor.stored_line_position = cursor.head.x;
        }
    }

    //TODO: remove unnecessary cursors
    pub fn move_cursors_document_start(&mut self){
        for cursor in self.cursors.iter_mut(){
            *cursor = Cursor::default();
            cursor.stored_line_position = cursor.head.x;
        }
    }

    //TODO: remove unnecessary cursors
    pub fn move_cursors_document_end(&mut self){
        let document_length = self.len();
        for cursor in self.cursors.iter_mut(){
            cursor.set_both_y(document_length.saturating_sub(1));
        
            let line = match self.lines.get(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            let line_width = line.graphemes(true).count();

            cursor.set_both_x(line_width);
            cursor.stored_line_position = cursor.head.x;
        }
    }

    pub fn extend_selection_right(&mut self){
        let document_length = self.len();
        for cursor in self.cursors.iter_mut(){
            let line = match self.lines.get(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            let line_width = line.graphemes(true).count();
            if cursor.head.x < line_width{
                cursor.head.x = cursor.head.x.saturating_add(1);
            }
            else if cursor.head.y < document_length.saturating_sub(1){
                cursor.head.y = cursor.head.y.saturating_add(1);
                cursor.head.x = 0;
            }
            cursor.stored_line_position = cursor.head.x;
        }
    }

    pub fn extend_selection_left(&mut self){
        for cursor in self.cursors.iter_mut(){
            let line = match self.lines.get(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            if cursor.head.x > 0{
                cursor.head.x = cursor.head.x.saturating_sub(1);
            }
            else if cursor.head.y > 0{
                cursor.head.y = cursor.head.y.saturating_sub(1);
                cursor.head.x = line.graphemes(true).count()
            }
            cursor.stored_line_position = cursor.head.x;
        }
    }

    pub fn extend_selection_up(&mut self){
        for cursor in self.cursors.iter_mut(){
            cursor.head.y = cursor.head.y.saturating_sub(1);
        }
        self.clamp_selection_cursors_to_line_end();
    }

    pub fn extend_selection_down(&mut self){
        let document_length = self.len();
        for cursor in self.cursors.iter_mut(){
            if cursor.head.y < document_length.saturating_sub(1){
                cursor.head.y = cursor.head.y.saturating_add(1);
            }
        }
        self.clamp_selection_cursors_to_line_end();
    }

    pub fn extend_selection_home(&mut self){
        for cursor in self.cursors.iter_mut(){
            let line = match self.lines.get(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            let start_of_line = get_first_non_whitespace_character_index(line);
            if cursor.head.x == start_of_line{
                cursor.head.x = 0;
            }else{
                cursor.head.x = start_of_line;
            }
            cursor.stored_line_position = cursor.head.x;
        }
    }

    pub fn extend_selection_end(&mut self){
        for cursor in self.cursors.iter_mut(){
            let line = match self.lines.get(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            let line_width = line.graphemes(true).count();
            cursor.head.x = line_width;
            cursor.stored_line_position = cursor.head.x;
        }
    }

    pub fn _extend_selection_page_up(&mut self){}

    pub fn _extend_selection_page_down(&mut self){}

    pub fn collapse_selection_cursors(&mut self){
        for cursor in self.cursors.iter_mut(){
            if cursor.head.y == cursor.anchor.y{
                cursor.anchor.x = cursor.stored_line_position;
                cursor.head.x = cursor.stored_line_position;
            }else{
                cursor.anchor.x = cursor.stored_line_position;
                cursor.head.x = cursor.stored_line_position;
                cursor.anchor.y = cursor.head.y;
            }
        }
    }

    fn clamp_cursor_to_line_end(cursor: &mut Cursor, lines: &Vec<String>){
        let line = match lines.get(cursor.head.y){
            Some(line) => line,
            None => panic!("No line at cursor position. This should be impossible")
        };
        let line_width = line.graphemes(true).count();
        cursor.set_both_x(
            if cursor.head.x > line_width || cursor.stored_line_position > line_width{
                line_width
            }else{
                cursor.stored_line_position
            }
        );
    }

    fn clamp_selection_cursors_to_line_end(&mut self){
        for cursor in self.cursors.iter_mut(){
            let line = match self.lines.get(cursor.head.y){
                Some(line) => line,
                None => panic!("No line at cursor position. This should be impossible")
            };
            let line_width = line.graphemes(true).count();
            
            cursor.head.x = if cursor.head.x > line_width
                            || cursor.stored_line_position > line_width
            {
                line_width
            }else{
                cursor.stored_line_position
            };
        }
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
        //TODO: remove unnecessary cursors
        let cursor = self.cursors.last().unwrap();
        if line_number < self.len(){
            //self.cursor_head.y = line_number;
            //self.cursor_anchor.y = line_number;
            //self.clamp_cursor_to_line_end();
            self.set_cursor_position(Position::new(cursor.stored_line_position, line_number));
            //Ok(())
        }
        //else{
        //    Err(())
        //}
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

    // returns the number of lines in this document
    pub fn len(&self) -> usize{
        self.lines.len()
    }
    pub fn is_empty(&self) -> bool{
        self.lines.is_empty()
    }

    pub fn scroll_client_view_down(&mut self, amount: usize){
        if self.client_view.vertical_start + self.client_view.height + amount <= self.len(){
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
    fn verify_set_cursor_position_behavior(){
        let mut doc = Document::default();
        doc.lines = vec!["1234".to_string(), "1".to_string(), "123".to_string()];

        // setting y inside doc should work
        doc.set_cursor_position(Position::new(0, 2));
        assert!(doc.cursors.last().unwrap().head.y() == 2);
        assert!(doc.cursors.last().unwrap().head.x() == 0);
        // setting y past doc end does nothing
        doc.set_cursor_position(Position::new(0, 3));
        assert!(doc.cursors.last().unwrap().head.y() == 2);
        assert!(doc.cursors.last().unwrap().head.x() == 0);
        // setting x past line end should restrict x to line end
        doc.set_cursor_position(Position::new(4, 2));
        assert!(doc.cursors.last().unwrap().head.y() == 2);
        assert!(doc.cursors.last().unwrap().head.x() == 3);
    }

    //add cursor on line above
    #[test]
    fn add_cursor_on_line_above_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        doc.set_cursor_position(Position::new(9, 1));
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

    //enter
        //also test auto indent
    //insert newline
    #[test]
    fn insert_newline_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.enter();
        println!("{:?}", doc.lines);
        assert!(doc.lines == vec!["".to_string(), "idk".to_string()]);
        assert!(doc.cursors.last().unwrap().head.x() == 0);
        assert!(doc.cursors.last().unwrap().head.y() == 1);
    }
    #[test]
    fn auto_indent_works(){
        assert!(false);
    }
    
    //insert char
    #[test]
    fn insert_char_works(){
        let mut doc = Document::default();
        doc.lines = vec!["dk".to_string()];

        doc.insert_char_at_cursors('i');
        assert!(doc.lines == vec!["idk".to_string()]);
        assert!(doc.cursors.last().unwrap().head.x() == 1);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
    }
    //tab
    #[test]
    fn tab_insert_tab_width_spaces(){
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
    //delete
        //delete next char
        //delete next newline
    #[test]
    fn delete_removes_character(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.delete();
        assert!(doc.lines == vec!["dk".to_string()]);
        assert!(doc.cursors.last().unwrap().head.x() == 0);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
    }
    #[test]
    fn delete_at_end_of_line_appends_next_line_to_current(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        doc.set_cursor_position(Position::new(3, 0));
        doc.delete();
        assert!(doc.lines == vec!["idksomething".to_string()]);
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
    }
    #[test]
    fn delete_removes_selection(){
        assert!(false);
    }
    
    //backspace
        //delete prev char
        //delete prev newline
        //delete prev tab
    #[test]
    fn backspace_removes_previous_character(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.set_cursor_position(Position::new(1, 0));
        doc.backspace();
        println!("{:?}", doc.lines);
        assert!(doc.lines == vec!["dk".to_string()]);
        println!("{:?}", doc.cursors.last().unwrap().head);
        assert!(doc.cursors.last().unwrap().head.x() == 0);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
    }
    #[test]
    fn backspace_at_start_of_line_appends_current_line_to_end_of_previous_line(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        doc.set_cursor_position(Position::new(0, 1));
        doc.backspace();
        println!("{:?}", doc.lines);
        assert!(doc.lines == vec!["idksomething".to_string()]);
        println!("{:?}", doc.cursors.last().unwrap().head);
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
    }
    #[test]
    fn backspace_removes_previous_tab(){
        let mut doc = Document::default();
        let mut line = String::new();
        for _ in 0..TAB_WIDTH{
            line.push(' ');
        }
        line.push_str("something");
        doc.lines = vec![line];

        doc.set_cursor_position(Position::new(TAB_WIDTH, 0));
        doc.backspace();
        println!("{:?}", doc.lines);
        assert!(doc.lines == vec!["something".to_string()]);
        println!("{:?}", doc.cursors.last().unwrap().head);
        assert!(doc.cursors.last().unwrap().head.x() == 0);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
    }
    
    #[test]
    fn move_cursor_left_at_document_start_does_not_move_cursor(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.move_cursors_left();
        assert!(doc.cursors.last().unwrap().head.y == 0);
        assert!(doc.cursors.last().unwrap().head.x == 0);
    }
    #[test]
    fn move_cursor_left_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.set_cursor_position(Position::new(1, 0));
        assert!(doc.cursors.last().unwrap().head.y == 0);
        assert!(doc.cursors.last().unwrap().head.x == 1);
        doc.move_cursors_left();
        assert!(doc.cursors.last().unwrap().head.y == 0);
        assert!(doc.cursors.last().unwrap().head.x == 0);
    }
    #[test]
    fn move_cursor_left_at_line_start_moves_cursor_to_previous_line_end(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        doc.set_cursor_position(Position::new(0, 1));
        assert!(doc.cursors.last().unwrap().head.y == 1);
        assert!(doc.cursors.last().unwrap().head.x == 0);
        doc.move_cursors_left();
        assert!(doc.cursors.last().unwrap().head.y == 0);
        assert!(doc.cursors.last().unwrap().head.x == 3);
    }
    
    #[test]
    fn move_cursor_up_at_document_start_does_not_move_cursor(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.move_cursors_up();
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().head.x() == 0);
    }
    #[test]
    fn move_cursor_up_works_when_moving_to_shorter_line(){
        let mut doc = Document::default();
        doc.lines = vec!["1".to_string(), "123".to_string()];

        let position = Position::new(3, 1);
        doc.set_cursor_position(position);
        assert!(doc.cursors.last().unwrap().head.y() == 1);
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        doc.move_cursors_up();
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().head.x() == 1);
    }
    #[test]
    fn move_cursor_up_works_when_moving_to_longer_line(){
        let mut doc = Document::default();
        doc.lines = vec!["1234".to_string(), "1".to_string()];

        let position = Position::new(1, 1);
        doc.set_cursor_position(position);
        assert!(doc.cursors.last().unwrap().head.y() == 1);
        assert!(doc.cursors.last().unwrap().head.x() == 1);
        doc.move_cursors_up();
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().head.x() == 1);
    }

    #[test]
    fn move_cursor_right_at_document_end_does_not_move_cursor(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.set_cursor_position(Position::new(3, 0));
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        doc.move_cursors_right();
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().head.x() == 3);
    }
    #[test]
    fn move_cursor_right_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.move_cursors_right();
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().head.x() == 1);
    }
    #[test]
    fn move_cursor_right_at_line_end_moves_cursor_to_start_of_next_line(){
        let mut doc = Document::default();
        doc.lines = vec!["1".to_string(), "2".to_string()];

        doc.set_cursor_position(Position::new(1, 0));
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().head.x() == 1);
        doc.move_cursors_right();
        assert!(doc.cursors.last().unwrap().head.y() == 1);
        assert!(doc.cursors.last().unwrap().head.x() == 0);
    }

    #[test]
    fn move_cursor_down_at_document_end_does_not_move_cursor(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.set_cursor_position(Position::new(3, 0));
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        doc.move_cursors_down();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().head.x() == 3);
    }
    #[test]
    fn move_cursor_down_works_when_moving_to_shorter_line(){
        let mut doc = Document::default();
        doc.lines = vec!["123".to_string(), "1".to_string()];

        let position = Position::new(3, 0);
        doc.set_cursor_position(position);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        doc.move_cursors_down();
        assert!(doc.cursors.last().unwrap().head.y() == 1);
        assert!(doc.cursors.last().unwrap().head.x() == 1);
    }
    #[test]
    fn move_cursor_down_works_when_moving_to_longer_line(){
        let mut doc = Document::default();
        doc.lines = vec!["1".to_string(), "1234".to_string()];

        let position = Position::new(1, 0);
        doc.set_cursor_position(position);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().head.x() == 1);
        doc.move_cursors_down();
        assert!(doc.cursors.last().unwrap().head.y() == 1);
        assert!(doc.cursors.last().unwrap().head.x() == 1);
    }

    //move cursors page up
    //move cursors page down
    //move cursors home
    //move cursors end
    //move cursor doc start
    //move cursor doc end

    //extend selection right
    #[test]
    fn extend_selection_right_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.extend_selection_right();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 1);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 0);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }
    #[test]
    fn extend_selection_right_at_end_of_line_wraps_to_next_line(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        doc.set_cursor_position(Position::new(3, 0));
        doc.extend_selection_right();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 0);
        assert!(doc.cursors.last().unwrap().head.y() == 1);
        assert!(doc.cursors.last().unwrap().anchor.x() == 3);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }
    #[test]
    fn extend_selection_right_at_document_end_does_not_extend_selection(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.set_cursor_position(Position::new(3, 0));
        doc.extend_selection_right();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 3);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }

    //extend selection left
    #[test]
    fn extend_selection_left_works(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.set_cursor_position(Position::new(3, 0));
        doc.extend_selection_left();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 2);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 3);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }
    #[test]
    fn extend_selection_left_at_start_of_line_wraps_to_previous_line(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        doc.set_cursor_position(Position::new(0, 1));
        doc.extend_selection_left();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 9);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 0);
        assert!(doc.cursors.last().unwrap().anchor.y() == 1);
    }
    #[test]
    fn extend_selection_left_at_document_start_does_not_extend_selection(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.extend_selection_left();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 0);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 0);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }

    //extend selection up
    #[test]
    fn extend_selection_up_works_when_previous_line_is_shorter(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        doc.set_cursor_position(Position::new(9, 1));
        doc.extend_selection_up();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 9);
        assert!(doc.cursors.last().unwrap().anchor.y() == 1);
    }
    #[test]
    fn extend_selection_up_works_when_previous_line_is_longer(){
        let mut doc = Document::default();
        doc.lines = vec!["something".to_string(), "idk".to_string()];

        doc.set_cursor_position(Position::new(3, 1));
        doc.extend_selection_up();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 3);
        assert!(doc.cursors.last().unwrap().anchor.y() == 1);
    }
    #[test]
    fn extend_selection_up_at_document_start_does_not_extend_selection(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.extend_selection_up();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 0);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 0);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }
    
    //extend selection down
    #[test]
    fn extend_selection_down_works_when_next_line_shorter(){
        let mut doc = Document::default();
        doc.lines = vec!["something".to_string(), "idk".to_string()];

        doc.set_cursor_position(Position::new(9, 0));
        doc.extend_selection_down();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        assert!(doc.cursors.last().unwrap().head.y() == 1);
        assert!(doc.cursors.last().unwrap().anchor.x() == 9);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }
    #[test]
    fn extend_selection_down_works_when_next_line_longer(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        doc.set_cursor_position(Position::new(3, 0));
        doc.extend_selection_down();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        assert!(doc.cursors.last().unwrap().head.y() == 1);
        assert!(doc.cursors.last().unwrap().anchor.x() == 3);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }
    #[test]
    fn extend_selection_down_at_document_end_does_not_extend_selection(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string()];

        doc.set_cursor_position(Position::new(3, 0));
        doc.extend_selection_down();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 3);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }
    
    //extend selection home
    //extend selection end
    //extend selection page up
    //extend selection page down
    
    //collapse selection cursors
        //when on same line and head less than anchor
    #[test]
    fn collapse_selection_cursors_works_when_on_same_line_and_head_less_than_anchor(){
        let mut doc = Document::default();
        doc.lines = vec!["something".to_string()];

        doc.set_cursor_position(Position::new(9, 0));
        doc.extend_selection_left();
        doc.collapse_selection_cursors();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 8);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 8);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }
        //when on same line and anchor less than head
    #[test]
    fn collapse_selection_cursors_works_when_on_same_line_and_anchor_less_than_head(){
        let mut doc = Document::default();
        doc.lines = vec!["something".to_string()];

        doc.extend_selection_right();
        doc.collapse_selection_cursors();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 1);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 1);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }
        //when on different lines and head less than anchor
    #[test]
    fn collapse_selection_cursors_works_when_on_different_lines_and_head_less_than_anchor(){
        let mut doc = Document::default();
        doc.lines = vec!["something".to_string(), "idk".to_string()];

        doc.set_cursor_position(Position::new(3, 1));
        doc.extend_selection_up();
        doc.collapse_selection_cursors();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        assert!(doc.cursors.last().unwrap().head.y() == 0);
        assert!(doc.cursors.last().unwrap().anchor.x() == 3);
        assert!(doc.cursors.last().unwrap().anchor.y() == 0);
    }
        //when on different lines and anchor less than head
    #[test]
    fn collapse_selection_cursors_works_when_on_different_lines_and_anchor_less_than_head(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "something".to_string()];

        doc.set_cursor_position(Position::new(3, 0));
        doc.extend_selection_down();
        doc.collapse_selection_cursors();
        println!("{:?}", doc.cursors.last().unwrap());
        assert!(doc.cursors.last().unwrap().head.x() == 3);
        assert!(doc.cursors.last().unwrap().head.y() == 1);
        assert!(doc.cursors.last().unwrap().anchor.x() == 3);
        assert!(doc.cursors.last().unwrap().anchor.y() == 1);
    }

    //clamp cursors to line end (verified in cursor movement tests)
    //clamp selection cursors to line end (verified in extend selection tests)

    //goto

    #[test]
    fn len_returns_last_line_number(){
        let mut doc = Document::default();
        doc.lines = vec!["idk".to_string(), "some".to_string(), "shit".to_string()];
        assert!(doc.len() == 3);
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
