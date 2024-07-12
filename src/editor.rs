use crate::document::Document;
use std::{collections::HashMap, error::Error};



pub struct Editor{
    documents: HashMap<String, Document>,
}
impl Default for Editor{
    fn default() -> Self {
        Self{
            documents: HashMap::new(),
        }
    }
}
impl Editor{
    pub fn document(&self, client_address: &str) -> Option<&Document>{
        if let Some(doc) = self.documents.get(client_address){
            return Some(&doc);
        }

        None
    }
    pub fn document_mut(&mut self, client_address: &str) -> Option<&mut Document>{
        if let Some(doc) = self.documents.get_mut(client_address){
            return Some(doc);
        }

        None
    }
    pub fn open_document(&mut self, path: &str, client_address: &str) -> Result<(), Box<dyn Error>>{
        let doc = Document::open(path)?;
        self.documents.insert(client_address.to_string(), doc);

        Ok(())
    }
    pub fn close_document(&mut self, client_address: &str){
        if self.documents.get(client_address).is_some(){
            self.documents.remove(client_address);
        }
    }
}
