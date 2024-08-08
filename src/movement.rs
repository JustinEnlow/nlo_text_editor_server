use crate::document;
use crate::View;
use crate::selection::Selection;
use ropey::{Rope, RopeSlice};

/// Sets rope cursor to a 0 based line number
pub fn set_rope_cursor_position_from_line_number(mut selection: Selection, line_number: usize, text: RopeSlice) -> Selection{
    if line_number < text.len_lines(){ //is len lines 1 based?
        let start_of_line = text.line_to_char(line_number);
        let line = text.line(line_number);
        let line_width = document::line_width_excluding_newline(line);
        if selection.stored_line_position() < line_width{
            selection.set_anchor(start_of_line + selection.stored_line_position());
            selection.set_head(start_of_line + selection.stored_line_position());
        }else{
            selection.set_anchor(start_of_line + line_width);
            selection.set_head(start_of_line + line_width);
        }
    }

    selection
}

pub fn move_cursor_up(mut selection: Selection, text: RopeSlice) -> Selection{
    assert!(selection.head() == selection.anchor());    //or selection.head() == selection.anchor() + 1, if block cursor semantics
    let line_number = text.char_to_line(selection.head());
    let previous_line_number = line_number.saturating_sub(1);
    selection = set_rope_cursor_position_from_line_number(selection, previous_line_number, text.slice(..));

    selection
}

pub fn move_cursor_down(mut selection: Selection, text: RopeSlice) -> Selection{
    let line_number = text.char_to_line(selection.head());
    let next_line_number = line_number.saturating_add(1);
    selection = set_rope_cursor_position_from_line_number(selection, next_line_number, text);

    selection
}

pub fn move_cursor_right(mut selection: Selection, text: RopeSlice) -> Selection{
    if selection.head().saturating_add(1) < text.len_chars()
    || selection.head().saturating_add(1) == text.len_chars(){
        selection.set_head(selection.head().saturating_add(1));
        selection.set_anchor(selection.anchor().saturating_add(1));
        let line_start = text.line_to_char(text.char_to_line(selection.head()));
        selection.set_stored_line_position(selection.head().saturating_sub(line_start));
    }

    selection
}

pub fn move_cursor_left(mut selection: Selection, text: RopeSlice) -> Selection{
    selection.set_head(selection.head().saturating_sub(1));
    selection.set_anchor(selection.anchor().saturating_sub(1));
    let line_start = text.line_to_char(text.char_to_line(selection.head()));
    selection.set_stored_line_position(selection.head().saturating_sub(line_start));

    selection
}

pub fn move_cursor_page_up(mut selection: Selection, text: RopeSlice, client_view: View) -> Selection{
    let line_number = text.char_to_line(selection.head());
    let goal_line_number = line_number.saturating_sub(client_view.height.saturating_sub(1));
    selection = set_rope_cursor_position_from_line_number(selection, goal_line_number, text);

    selection
}

pub fn move_cursor_page_down(mut selection: Selection, text: RopeSlice, client_view: View) -> Selection{
    let document_length = text.len_lines();
    let line_number = text.char_to_line(selection.head());
    let goal_line_number = if line_number.saturating_add(client_view.height) <= document_length{
        line_number.saturating_add(client_view.height.saturating_sub(1))
    }else{
        document_length.saturating_sub(1)
    };
    selection = set_rope_cursor_position_from_line_number(selection, goal_line_number, text);

    selection
}

pub fn move_cursor_home(mut selection: Selection, text: RopeSlice) -> Selection{
    let line_number = text.char_to_line(selection.head());
    let line_start = text.line_to_char(line_number);
    let text_start_offset = document::get_first_non_whitespace_character_index(text.line(line_number));
    let text_start = line_start.saturating_add(text_start_offset);

    if selection.head() == text_start{
        //TODO: break out into own move_cursor_line_start fn?
        selection.set_head(line_start);
        selection.set_anchor(line_start);
    }else{
        //TODO: break out into own move_cursor_line_text_start fn?
        selection.set_head(text_start);
        selection.set_anchor(text_start);
    }
    selection.set_stored_line_position(selection.head().saturating_sub(line_start));

    selection
}

pub fn move_cursor_end(mut selection: Selection, text: RopeSlice) -> Selection{
    let line_number = text.char_to_line(selection.head());
    let line = text.line(line_number);
    let line_width = document::line_width_excluding_newline(line);
    let line_start = text.line_to_char(line_number);
    let line_end = line_start.saturating_add(line_width);

    selection.set_head(line_end);
    selection.set_anchor(line_end);
    selection.set_stored_line_position(line_end.saturating_sub(line_start));

    selection
}

pub fn move_cursor_document_start(mut selection: Selection) -> Selection{
    selection.set_head(0);
    selection.set_anchor(0);
    selection.set_stored_line_position(0);

    selection
}

pub fn move_cursor_document_end(mut selection: Selection, text: RopeSlice) -> Selection{
    selection.set_head(text.len_chars());
    selection.set_anchor(text.len_chars());
    let line_start = text.line_to_char(text.char_to_line(selection.head()));
    selection.set_stored_line_position(text.len_chars().saturating_sub(line_start));

    selection
}

pub fn extend_selection_right(mut selection: Selection, text: RopeSlice) -> Selection{
    if selection.head().saturating_add(1) < text.len_chars()
    || selection.head().saturating_add(1) == text.len_chars()
    {
        selection.set_head(selection.head().saturating_add(1));
        let line_start = text.line_to_char(text.char_to_line(selection.head()));
        selection.set_stored_line_position(selection.head().saturating_sub(line_start));
    }

    selection
}

pub fn extend_selection_left(mut selection: Selection, text: RopeSlice) -> Selection{
    selection.set_head(selection.head().saturating_sub(1));
    let line_start = text.line_to_char(text.char_to_line(selection.head()));
    selection.set_stored_line_position(selection.head().saturating_sub(line_start));

    selection
}

pub fn extend_selection_up(mut selection: Selection, text: RopeSlice) -> Selection{
    let line_number = text.char_to_line(selection.head());
    let previous_line_number = line_number.saturating_sub(1);
    if previous_line_number < text.len_lines(){
        let start_of_line = text.line_to_char(previous_line_number);
        let line = text.line(previous_line_number);
        let line_width = document::line_width_excluding_newline(line);
        if selection.stored_line_position() < line_width{
            selection.set_head(start_of_line + selection.stored_line_position());
        }else{
            selection.set_head(start_of_line + line_width);
        }
    }

    selection
}

pub fn extend_selection_down(mut selection: Selection, text: RopeSlice) -> Selection{
    let line_number = text.char_to_line(selection.head());
    let next_line_number = line_number.saturating_add(1);
    if next_line_number < text.len_lines(){
        let start_of_line = text.line_to_char(next_line_number);
        let line = text.line(next_line_number);
        let line_width = document::line_width_excluding_newline(line);
        if selection.stored_line_position() < line_width{
            selection.set_head(start_of_line + selection.stored_line_position());
        }else{
            selection.set_head(start_of_line + line_width);
        }
    }

    selection
}

pub fn extend_selection_home(mut selection: Selection, text: RopeSlice) -> Selection{
    let line_number = text.char_to_line(selection.head());
    let line_start = text.line_to_char(line_number);
    let text_start_offset = document::get_first_non_whitespace_character_index(text.line(line_number));
    let text_start = line_start.saturating_add(text_start_offset);

    if selection.head() == text_start{
        selection.set_head(line_start);
    }else{
        selection.set_head(text_start);
    }
    selection.set_stored_line_position(selection.head().saturating_sub(line_start));

    selection
}

pub fn extend_selection_end(mut selection: Selection, text: RopeSlice) -> Selection{
    let line_number = text.char_to_line(selection.head());
    let line = text.line(line_number);
    let line_width = document::line_width_excluding_newline(line);
    let line_start = text.line_to_char(line_number);
    let line_end = line_start.saturating_add(line_width);

    selection.set_head(line_end);
    selection.set_stored_line_position(line_end.saturating_sub(line_start));

    selection
}

pub fn collapse_selection_cursor(mut selection: Selection) -> Selection{
    selection.set_anchor(selection.head());

    selection
}





//SET ROPE CURSOR POSITION
#[test]
fn set_rope_cursor_position_works_when_desired_position_is_inside_doc_boundaries(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::default();    //[]idk\nsomething\nelse
    let expected_rope_cursor = Selection::new(14, 14, 0);   //idk\nsomething\n[]else
    let line_number: usize = 2;//3;
    rope_cursor = set_rope_cursor_position_from_line_number(rope_cursor, line_number, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn set_rope_cursor_position_should_do_nothing_when_desired_line_number_is_greater_than_doc_length(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::default();    //[]idk\nsomething\nelse
    let expected_rope_cursor = Selection::default();   //[]idk\nsomething\nelse
    let line_number: usize = 5;//6;
    rope_cursor = set_rope_cursor_position_from_line_number(rope_cursor, line_number, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn set_rope_cursor_position_restricts_cursor_to_line_end_when_cursor_stored_line_position_is_greater_than_line_width(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::new(13, 13, 9);    //idk\nsomething[]\nelse
    let expected_rope_cursor = Selection::new(3, 3, 9); //idk[]\nsomething\nelse
    let line_number: usize = 0;//1;
    rope_cursor = set_rope_cursor_position_from_line_number(rope_cursor, line_number, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//MOVE CURSOR LEFT
#[test]
fn move_cursor_left_at_document_start_does_not_move_cursor(){
    let text = Rope::from("idk\nsomething\nelse");  //TODO: set up a better text to use, specific to move left
    let mut rope_cursor = Selection::default();
    let expected_rope_cursor = Selection::default();
    rope_cursor = move_cursor_left(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn move_left_works(){
    let text = Rope::from("idk\nsomething\nelse");  //TODO: set up a better text to use, specific to move left
    let mut rope_cursor = Selection::new(2, 2, 2);
    let expected_rope_cursor = Selection::new(1, 1, 1);
    rope_cursor = move_cursor_left(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn move_cursor_left_at_start_of_line_resets_stored_line_position(){
    let text = Rope::from("idk\nsomething\nelse");  //TODO: set up a better text to use, specific to move left
    let mut rope_cursor = Selection::new(4, 4, 0);  //idk\n[]something\nelse
    let expected_rope_cursor = Selection::new(3, 3, 3); //idk[]\nsomething\nelse
    rope_cursor = move_cursor_left(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//MOVE CURSOR UP
#[test]
fn move_cursor_up_at_document_start_does_not_move_cursor(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::default();
    let expected_rope_cursor = Selection::default();
    rope_cursor = move_cursor_up(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn move_cursor_up_works_when_moving_to_shorter_line(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::new(13, 13, 9);    //idk\nsomething[]\nelse
    let expected_rope_cursor = Selection::new(3, 3, 9); //idk[]\nsomething\nelse
    rope_cursor = move_cursor_up(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn move_cursor_up_works_when_moving_to_longer_line(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::new(18, 18, 4);    //idk\nsomething\nelse[]
    let expected_rope_cursor = Selection::new(8, 8, 4); //idk\nsome[]thing\nelse
    rope_cursor = move_cursor_up(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//MOVE CURSOR RIGHT
#[test]
fn move_cursor_right_at_document_end_does_not_move_cursor(){
    let text = Rope::from("012\n");
    let mut rope_cursor = Selection::new(4, 4, 0);  //012\n[]
    let expected_rope_cursor = Selection::new(4, 4, 0); //012\n[]
    rope_cursor = move_cursor_right(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);

}
#[test]
fn move_cursor_right_works(){
    let text = Rope::from("012\n");
    let mut rope_cursor = Selection::default();    //[]012\n
    let expected_rope_cursor = Selection::new(1, 1, 1); //0[]12\n
    rope_cursor = move_cursor_right(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn move_cursor_right_at_end_of_line_resets_stored_line_position(){
    let text = Rope::from("012\n0");
    let mut rope_cursor = Selection::new(3, 3, 3);  //012[]\n0
    let expected_rope_cursor = Selection::new(4, 4, 0); //012\n[]0
    rope_cursor = move_cursor_right(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//MOVE CURSOR DOWN
#[test]
fn move_cursor_down_at_document_end_does_not_move_cursor(){
    let text = Rope::from("012\n");
    let mut rope_cursor = Selection::new(4, 4, 0);  //012\n[]
    let expected_rope_cursor = Selection::new(4, 4, 0); //012\n[]
    rope_cursor = move_cursor_down(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn move_cursor_down_works_when_moving_to_shorter_line(){
    let text = Rope::from("012\n0");
    let mut rope_cursor = Selection::new(3, 3, 3);  //012[]\n0
    let expected_rope_cursor = Selection::new(5, 5, 3); //012\n0[]
    rope_cursor = move_cursor_down(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn move_cursor_down_works_when_moving_to_longer_line(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::new(3, 3, 3);  //idk[]\nsomething\nelse
    let expected_rope_cursor = Selection::new(7, 7, 3); //idk\nsom[]ething\nelse
    rope_cursor = move_cursor_down(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//MOVE CURSOR PAGE UP
#[test]
fn move_cursor_page_up_works(){
    let text = Rope::from("idk\nsomething\nelse");
    let client_view = View{horizontal_start: 0, vertical_start: 0, width: 2, height: 2};
    let mut rope_cursor = Selection::new(6, 6, 2);  //idk\nso[]mething\nelse
    let expected_rope_cursor = Selection::new(2, 2, 2); //id[]k\nsomething\nelse
    rope_cursor = move_cursor_page_up(rope_cursor, text.slice(..), client_view);
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
//MOVE CURSOR PAGE DOWN
#[test]
fn move_cursor_page_down_works(){
    let text = Rope::from("idk\nsomething\nelse");
    let client_view = View{horizontal_start: 0, vertical_start: 0, width: 2, height: 2};
    let mut rope_cursor = Selection::default();  //[]idk\nsomething\nelse
    let expected_rope_cursor = Selection::new(4, 4, 0); //idk\n[]something\nelse
    rope_cursor = move_cursor_page_down(rope_cursor, text.slice(..), client_view);
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//MOVE CURSOR HOME
#[test]
fn move_cursor_home_moves_cursor_to_text_start_when_cursor_past_text_start(){
    let text = Rope::from("    idk\n");
    let mut rope_cursor = Selection::new(6, 6, 6);  //id[]k\n
    let expected_rope_cursor = Selection::new(4, 4, 4); //    []idk\n
    rope_cursor = move_cursor_home(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn move_cursor_home_moves_cursor_to_line_start_when_cursor_at_text_start(){
    let text = Rope::from("    idk\n");
    let mut rope_cursor = Selection::new(4, 4, 4);  //    []idk\n
    let expected_rope_cursor = Selection::default();   //[]    idk\n
    rope_cursor = move_cursor_home(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn move_cursor_home_moves_cursor_to_text_start_when_cursor_before_text_start(){
    let text = Rope::from("    idk\n");
    let mut rope_cursor = Selection::new(1, 1, 1);  // []   idk\n
    let expected_rope_cursor = Selection::new(4, 4, 4); //    []idk\n
    rope_cursor = move_cursor_home(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
//TODO: should there be another test to verify stored line position functionality in multiline texts?

//MOVE CURSOR END
#[test]
fn move_cursor_end_moves_cursor_to_line_end(){
    let text = Rope::from("idk\n");
    let mut rope_cursor = Selection::default();    //[]idk\n
    let expected_rope_cursor = Selection::new(3, 3, 3); //idk[]\n
    rope_cursor = move_cursor_end(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//MOVE CURSOR DOC START
#[test]
fn move_cursor_doc_start_works(){
    let mut rope_cursor = Selection::new(12, 12, 12);
    let expected_rope_cursor = Selection::default();
    rope_cursor = move_cursor_document_start(rope_cursor);
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
//MOVE CURSOR DOC END
#[test]
fn move_cursor_document_end_works(){
    let text = Rope::from("idk\nsome\nshit");
    let mut rope_cursor = Selection::default();    //[]idk\nsome\nshit
    let expected_rope_cursor = Selection::new(13, 13, 4);   //idk\nsome\nshit[]
    rope_cursor = move_cursor_document_end(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//EXTEND SELECTION RIGHT
#[test]
fn extend_selection_right_at_document_end_does_not_extend_selection(){
    let text = Rope::from("012\n");
    let mut rope_cursor = Selection::new(4, 4, 0);  //012\n[]
    let expected_rope_cursor = Selection::new(4, 4, 0); //012\n[]
    rope_cursor = extend_selection_right(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);

}
#[test]
fn extend_selection_right_works(){
    let text = Rope::from("012\n");
    let mut rope_cursor = Selection::default();    //[]012\n
    let expected_rope_cursor = Selection::new(0, 1, 1); //[0]12\n
    rope_cursor = extend_selection_right(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn extend_selection_right_at_end_of_line_resets_stored_line_position(){
    let text = Rope::from("012\n0");
    let mut rope_cursor = Selection::new(3, 3, 3);  //012[]\n0
    let expected_rope_cursor = Selection::new(3, 4, 0); //012[\n]0
    rope_cursor = extend_selection_right(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
//TODO: test if selection extended left, extend selection right reduces selection

//EXTEND SELECTION LEFT
#[test]
fn extend_selection_left_at_document_start_does_not_extend_selection(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::default();
    let expected_rope_cursor = Selection::default();
    rope_cursor = extend_selection_left(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn extend_selection_left_works(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::new(2, 2, 2);  //id[]k\nsomething\nelse
    let expected_rope_cursor = Selection::new(2, 1, 1); //i]d[k\nsomething\nelse
    rope_cursor = extend_selection_left(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn extend_selection_left_at_start_of_line_resets_stored_line_position(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::new(4, 4, 0);  //idk\n[]something\nelse
    let expected_rope_cursor = Selection::new(4, 3, 3); //idk]\n[something\nelse
    rope_cursor = extend_selection_left(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
//TODO: test if selection extended right, extend selection left reduces selection

//EXTEND SELECTION UP
#[test]
fn extend_selection_up_at_document_start_does_not_extend_selection(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::default();
    let expected_rope_cursor = Selection::default();
    rope_cursor = extend_selection_up(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn extend_selection_up_works_when_moving_to_shorter_line(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::new(13, 13, 9);    //idk\nsomething[]\nelse
    let expected_rope_cursor = Selection::new(13, 3, 9);    //idk]\nsomething[\nelse
    rope_cursor = extend_selection_up(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn extend_selection_up_works_when_moving_to_longer_line(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::new(18, 18, 4);    //idk\nsomething\nelse[]
    let expected_rope_cursor = Selection::new(18, 8, 4);    //idk\nsome]thing\nelse[
    rope_cursor = extend_selection_up(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//EXTEND SELECTION DOWN
#[test]
fn extend_selection_down_at_document_end_does_not_extend_selection(){
    let text = Rope::from("012\n");
    let mut rope_cursor = Selection::new(4, 4, 0);  //012\n[]
    let expected_rope_cursor = Selection::new(4, 4, 0); //012\n[]
    rope_cursor = extend_selection_down(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn extend_selection_down_works_when_moving_to_shorter_line(){
    let text = Rope::from("012\n0");
    let mut rope_cursor = Selection::new(3, 3, 3);  //012[]\n0
    let expected_rope_cursor = Selection::new(3, 5, 3); //012[\n0]
    rope_cursor = extend_selection_down(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn extend_selection_down_works_when_moving_to_longer_line(){
    let text = Rope::from("idk\nsomething\nelse");
    let mut rope_cursor = Selection::new(3, 3, 3);  //idk[]\nsomething\nelse
    let expected_rope_cursor = Selection::new(3, 7, 3); //idk[\nsom]ething\nelse
    rope_cursor = extend_selection_down(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//EXTEND SELECTION HOME
#[test]
fn extend_selection_home_moves_cursor_to_text_start_when_cursor_past_text_start(){
    let text = Rope::from("    idk\n");
    let mut rope_cursor = Selection::new(6, 6, 6);  //    id[]k\n
    let expected_rope_cursor = Selection::new(6, 4, 4); //    ]id[k\n
    rope_cursor = extend_selection_home(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn extend_selection_home_moves_cursor_to_line_start_when_cursor_at_text_start(){
    let text = Rope::from("    idk\n");
    let mut rope_cursor = Selection::new(4, 4, 4);  //    []idk\n
    let expected_rope_cursor = Selection::new(4, 0, 0); //]    [idk\n
    rope_cursor = extend_selection_home(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}
#[test]
fn extend_selection_home_moves_cursor_to_text_start_when_cursor_before_text_start(){
    let text = Rope::from("    idk\n");
    let mut rope_cursor = Selection::new(1, 1, 1);  // []   idk\n
    let expected_rope_cursor = Selection::new(1, 4, 4); // [   ]idk\n
    rope_cursor = extend_selection_home(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//EXTEND SELECTION END
#[test]
fn extend_selection_end_moves_cursor_to_line_end(){
    let text = Rope::from("idk\n");
    let mut rope_cursor = Selection::default();    //[]idk\n
    let expected_rope_cursor = Selection::new(0, 3, 3); //[idk]\n
    rope_cursor = extend_selection_end(rope_cursor, text.slice(..));
    println!("expected: {expected_rope_cursor:?}\ngot: {rope_cursor:?}");
    assert!(rope_cursor == expected_rope_cursor);
}

//extend selection page up
//extend selection page down

//COLLAPSE SELECTION CURSOR
#[test]
fn collapse_selection_cursor_works_when_head_less_than_anchor(){
    let mut selection = Selection::new(5, 0, 0);
    let expected_selection = Selection::new(0, 0, 0);
    selection = collapse_selection_cursor(selection);
    assert!(selection == expected_selection);
}
#[test]
fn collapse_selection_cursor_works_when_head_greater_than_anchor(){
    let mut selection = Selection::new(0, 5, 5);
    let expected_selection = Selection::new(5, 5, 5);
    selection = collapse_selection_cursor(selection);
    assert!(selection == expected_selection);
}
