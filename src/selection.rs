use crate::Position;

/// 1 dimensional representation of a single selection(between anchor and head) within a text rope. a cursor is a selection with a anchor/head difference of 0 or 1(depending on cursor semantics)
#[derive(Default, PartialEq, Clone, Debug)]
pub struct Selection{
    /// the stationary portion of a selection
    anchor: usize,
    /// the mobile portion of a selection. this is the portion a user can extend to expand selection
    head: usize,
    /// the offset from the start of the line range.head is on
    stored_line_position: usize,
}
impl Selection{
    pub fn new(anchor: usize, head: usize, stored_line_position: usize) -> Self{
        Self{anchor, head, stored_line_position}
    }
    pub fn anchor(&self) -> usize{
        self.anchor
    }
    pub fn set_anchor(&mut self, to: usize){
        //TODO: figure out how to limit this to 0 <= to <= doc length in chars
        self.anchor = to;
    }
    pub fn head(&self) -> usize{
        self.head
    }
    pub fn set_head(&mut self, to: usize){
        //TODO: figure out how to limit this to 0 <= to <= doc length in chars
        self.head = to;
    }
    pub fn stored_line_position(&self) -> usize{
        self.stored_line_position
    }
    pub fn set_stored_line_position(&mut self, to: usize){
        //TODO: figure out how to limit this to 0 <= to <= doc length in chars
        self.stored_line_position = to;
    }
}



/// 2 dimensional representation of a single selection(between anchor and head) within document text
#[derive(Default, PartialEq, Debug)]
pub struct Selection2d{
    head: Position,
    anchor: Position,
}
impl Selection2d{
    pub fn new(head: Position, anchor: Position) -> Self{
        Self{
            head,
            anchor
        }
    }
    pub fn head(&self) -> &Position{
        &self.head
    }
    pub fn anchor(&self) -> &Position{
        &self.anchor
    }
}



//pub struct Selections{
//    selections: Vec<Selection>,
//    primary_selection_index: usize,
//}
//impl Selections{
//    pub fn pop(&mut self) -> Option<Selection>{
//        //TODO: figure out how to determine what to set primary_selection_index to
//        if self.selections.len() == 1{
//            None
//        }else{
//            self.selections.pop()
//        }
//    }
//    pub fn push(&mut self, selection: Selection){
//        self.selections.push(selection);
//        self.primary_selection_index = self.primary_selection_index + 1;
//    }
//    pub fn primary(&self) -> &Selection{
//        &self.selections[self.primary_selection_index]
//    }
//    pub fn last(&self) -> &Selection{
//        self.selections.last().unwrap()
//    }
//    //pub fn clear_non_primary_selections(cursors: &mut Vec<Selection>){
//    pub fn clear_non_primary_selections(&mut self){
//        //for x in (0..cursors.len()).rev(){
//        //    if x != 0{
//        //        cursors.pop();
//        //    }
//        //}
//        //for x in (0..self.selections.len()).rev(){
//        //    if x != 0{
//        //        self.selections.pop();
//        //    }
//        //}
//        for x in (0..self.selections.len()){
//            if x != self.primary_selection_index{
//                self.selections.pop();
//            }
//        }
//    }
//}
