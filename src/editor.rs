use crate::document::Document;
use std::{collections::HashMap, error::Error, path::PathBuf};



#[derive(Default)]
pub struct Editor{
    documents: HashMap<String, Document>,
}
impl Editor{
    pub fn document(&self, client_address: &str) -> Option<&Document>{
        if let Some(doc) = self.documents.get(client_address){
            return Some(doc);
        }

        None
    }
    pub fn document_mut(&mut self, client_address: &str) -> Option<&mut Document>{
        if let Some(doc) = self.documents.get_mut(client_address){
            return Some(doc);
        }

        None
    }
    pub fn open_document(&mut self, path: &PathBuf, client_address: &str) -> Result<(), Box<dyn Error>>{
        let doc = Document::open(path)?;
        self.documents.insert(client_address.to_string(), doc);

        Ok(())
    }
    pub fn close_document(&mut self, client_address: &str){
        if self.documents.contains_key(client_address){
            self.documents.remove(client_address);
        }
    }
}
