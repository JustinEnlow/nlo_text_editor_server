use crate::View;
use crate::Position;
use crate::document::Document;
use std::error::Error;



pub struct Editor{
    documents: Vec<Document>,
    focused_document_index: Option<usize>,
}
impl Default for Editor{
    fn default() -> Self {
        Self{
            documents: Vec::new(),
            focused_document_index: None
        }
    }
}
impl Editor{
    pub fn generate_buffer_for_current_view(&self, offset: Position, view: View) -> Vec<String>{
        let mut view_buffer = Vec::new();
        if let Some(doc) = self.document(){
            // if view is not detached from cursor
            view_buffer = doc.lines_in_bounds(
                doc.cursor_position().y() - offset.y(), 
                doc.cursor_position().y() - view.height, 
                doc.cursor_position().x() - offset.x(), 
                doc.cursor_position().x() - view.width
            )
        }
        view_buffer
    }
    pub fn documents(&self) -> &Vec<Document>{
        &self.documents
    }
    pub fn document(&self) -> Option<&Document>{
        if let Some(focused_document_index) = self.focused_document_index{
            return self.documents.get(focused_document_index);
        }

        None
    }
    pub fn document_mut(&mut self) -> Option<&mut Document>{
        if let Some(focused_document_index) = self.focused_document_index{
            return self.documents.get_mut(focused_document_index);
        }

        None
    }
    pub fn new_document(&mut self){
        self.documents.push(Document::default());
        self.focused_document_index = Some(self.documents.len().saturating_sub(1));
    }
    pub fn open_document(&mut self, path: &str) -> Result<(), Box<dyn Error>>{
        let doc = Document::open(path)?;
        self.documents.push(doc);
        self.focused_document_index = match self.focused_document_index{
            Some(idx) => Some(idx.saturating_add(1)),
            None => Some(0)
        };

        Ok(())
    }
    pub fn close_document(&mut self){
        if let Some(idx) = self.focused_document_index{
            self.documents.remove(idx);
            // reassign focused_document_index if possible
            if !self.documents.is_empty(){
                // is there a better way to determine which index to assign here?
                self.focused_document_index = Some(self.documents.len().saturating_sub(1));
            }else{
                self.focused_document_index = None;
            }
        }
    }
    pub fn increment_focused_document(&mut self){
        let new_index = match self.focused_document_index{
            Some(idx) => {
                // use conditional if we want increment to wrap
                if self.documents.len() > idx.saturating_add(1){
                    Some(idx.saturating_add(1))
                }else{Some(idx)/*Some(0)*/}
            }
            None => None
        };
        self.focused_document_index = new_index;
    }
    pub fn decrement_focused_document(&mut self){
        let new_index = match self.focused_document_index{
            Some(idx) => {
                // use conditional if we want decrement to wrap
                // if idx.saturating_sub(1) > 0{
                    Some(idx.saturating_sub(1))
                //}else{
                    //Some(self.documents.len().saturating_sub(1)) // sub 1?
                //}
            }
            None => None
        };
        self.focused_document_index = new_index;
    }
    pub fn focus_document_at_index(&mut self, index: usize){
        if index < self.documents.len(){
            self.focused_document_index = Some(index);
        }
    }
}
