/// This will be the main loop for the editor process/daemon(is this really a daemon, by definition?)
use nlo_text_editor_server::{editor::Editor, ServerAction};
use nlo_text_editor_server::ServerResponse;
use nlo_text_editor_server::MESSAGE_SIZE;
use std::{io::Write, net::{TcpListener, TcpStream}};
use std::io::Read;
use std::error::Error;


fn main(){
    // set up client/server stuff
    let listener = TcpListener::bind("127.0.0.1:7878").expect("failed to bind to port");
    println!("Server listening on port 7878\n");
    
    for stream in listener.incoming(){
        match stream{
            Ok(stream) => {
                std::thread::spawn(move ||{
                    let _ = handle_client(stream);
                });
            }
            Err(e) => {
                println!("Failed to establish connection: {}", e);
            }
        }
    }
}
    
fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn Error>>{
    //TODO: move editor into main, and handle multiple connections properly. we are starting a new editor session for each tcp connection, which
    //is exactly what we don't want
    let mut editor = Editor::default();

    let client_address = stream.peer_addr().unwrap();
    //TODO: return connection success response and assign client id?
    
    // loop and get requests
    let mut read_buffer = [0u8; MESSAGE_SIZE];
    loop{
        match stream.read(&mut read_buffer){
            Ok(size) => {
                // deserialize requests to actions, if possible
                let my_string = String::from_utf8_lossy(&read_buffer[0..size]);
                let action: ServerAction = match ron::from_str(&my_string){
                    Ok(action) => {action}
                    Err(e) => {return Err(Box::new(e));}
                };
                println!("server received: {:#?}", action);
                
                // perform requested action, if valid, and generate response
                if let Some(response) = server_action_to_response(action, &mut stream, &mut editor){
                    let serialized_response = ron::to_string(&response)?;
                    match stream.write(serialized_response.as_bytes()){
                        Ok(bytes_written) => {
                            if bytes_written == 0{} else {}
                        }
                        Err(e) => {return Err(Box::new(e));}
                    }
                    stream.flush().unwrap();
                    println!("server emitted: {:#?}\n", response);
                }
            }
            Err(e) => {
                println!("An error occurred. Terminating connection with {}. error: {}", client_address, e);
                //stream.shutdown(std::net::Shutdown::Both).unwrap();
                break;
            }
        }
    }

    Ok(())
}

fn server_action_to_response(action: ServerAction, stream: &mut TcpStream, editor: &mut Editor) -> Option<ServerResponse>{
    match action{
        ServerAction::Backspace => {
            if let Some(doc) = editor.document_mut(){
                doc.backspace();
                let _ = doc.scroll_view_following_cursor();
                Some(ServerResponse::DisplayView(
                    doc.get_client_view_text(), 
                    doc.get_client_view_line_numbers(),
                    doc.get_client_cursor_position(), 
                    doc.cursor_position(),
                    doc.is_modified(),
                ))
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        }
        ServerAction::CloseConnection => {
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            None
        },
        ServerAction::Delete => {
            if let Some(doc) = editor.document_mut(){
                doc.delete();
                let _ = doc.scroll_view_following_cursor();
                Some(ServerResponse::DisplayView(
                    doc.get_client_view_text(), 
                    doc.get_client_view_line_numbers(),
                    doc.get_client_cursor_position(), 
                    doc.cursor_position(),
                    doc.is_modified(),
                ))
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        }
        ServerAction::GoTo(line_number) => {
            if let Some(doc) = editor.document_mut(){
                if doc.go_to(line_number).is_ok(){
                    let _ = doc.scroll_view_following_cursor();
                    Some(ServerResponse::DisplayView(
                        doc.get_client_view_text(), 
                        doc.get_client_view_line_numbers(), 
                        doc.get_client_cursor_position(), 
                        doc.cursor_position(), 
                        doc.is_modified(),
                    ))
                }else{
                    Some(ServerResponse::Failed("could not go to line number".to_string()))
                }
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        }
        ServerAction::OpenFile(file) => {
            // i think we are trying to open files relative to our current(server) directory. CONFIRMED
            //TODO: open file with absolut paths
            match editor.open_document(/*&file*/file.to_str().expect("failed path buf to string")){
                Ok(_) => {
                    //Some(ServerResponse::Acknowledge)
                    if let Some(doc) = editor.document(){
                        Some(ServerResponse::FileOpened(doc.file_name(), doc.len()))
                    }else{
                        Some(ServerResponse::Failed("no document open".to_string()))
                    }
                }
                Err(e) => {
                    Some(ServerResponse::Failed(format!("{}", e)))
                }
            }
        },
        ServerAction::UpdateClientViewSize(width, height) => {
            if let Some(doc) = editor.document_mut(){
                doc.set_client_view_size(width as usize, height as usize);
                let _ = doc.scroll_view_following_cursor();
                Some(ServerResponse::DisplayView(
                    doc.get_client_view_text(), 
                    doc.get_client_view_line_numbers(), 
                    doc.get_client_cursor_position(), 
                    doc.cursor_position(),
                    doc.is_modified(),
                ))
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        //ServerAction::RequestClientViewText => {
        //    match editor.document(){
        //        Some(doc) => {
        //            Some(ServerResponse::DisplayView(doc.get_client_view_text()))
        //        }
        //        None => Some(ServerResponse::Failed("no document open".to_string()))
        //    }
        //},
        ServerAction::ScrollClientViewDown(amount) => {
            if let Some(doc) = editor.document_mut(){
                doc.scroll_client_view_down(amount);
                Some(ServerResponse::DisplayView(
                    doc.get_client_view_text(), 
                    doc.get_client_view_line_numbers(),
                    doc.get_client_cursor_position(), 
                    doc.cursor_position(),
                    doc.is_modified(),
                ))
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::ScrollClientViewLeft(amount) => {
            if let Some(doc) = editor.document_mut(){
                doc.scroll_client_view_left(amount);
                Some(ServerResponse::DisplayView(
                    doc.get_client_view_text(), 
                    doc.get_client_view_line_numbers(),
                    doc.get_client_cursor_position(), 
                    doc.cursor_position(),
                    doc.is_modified(),
                ))
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::ScrollClientViewRight(amount) => {
            if let Some(doc) = editor.document_mut(){
                doc.scroll_client_view_right(amount);
                Some(ServerResponse::DisplayView(
                    doc.get_client_view_text(), 
                    doc.get_client_view_line_numbers(),
                    doc.get_client_cursor_position(), 
                    doc.cursor_position(),
                    doc.is_modified(),
                ))
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::ScrollClientViewUp(amount) => {
            if let Some(doc) = editor.document_mut(){
                doc.scroll_client_view_up(amount);
                Some(ServerResponse::DisplayView(
                    doc.get_client_view_text(), 
                    doc.get_client_view_line_numbers(),
                    doc.get_client_cursor_position(), 
                    doc.cursor_position(),
                    doc.is_modified(),
                ))
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        //ServerAction::RequestClientCursorPosition => {
        //    if let Some(doc) = editor.document(){
        //        let client_cursor_position = doc.get_client_cursor_position();
        //        Some(ServerResponse::DisplayClientCursorPosition(client_cursor_position))
        //    }else{
        //        Some(ServerResponse::DisplayClientCursorPosition(None))
        //    }
        //},
        ServerAction::MoveCursorDocumentEnd => {
            if let Some(doc) = editor.document_mut(){
                doc.move_cursor_document_end();
                let should_update_client_view = doc.scroll_view_following_cursor();
                if should_update_client_view{
                    Some(ServerResponse::DisplayView(
                        doc.get_client_view_text(), 
                        doc.get_client_view_line_numbers(),
                        doc.get_client_cursor_position(), 
                        doc.cursor_position(),
                        doc.is_modified(),
                    ))
                }else{
                    Some(ServerResponse::CursorPosition(doc.get_client_cursor_position(), doc.cursor_position()))
                }
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        }
        ServerAction::MoveCursorDocumentStart => {
            if let Some(doc) = editor.document_mut(){
                doc.move_cursor_document_start();
                let should_update_client_view = doc.scroll_view_following_cursor();
                if should_update_client_view{
                    Some(ServerResponse::DisplayView(
                        doc.get_client_view_text(), 
                        doc.get_client_view_line_numbers(),
                        doc.get_client_cursor_position(), 
                        doc.cursor_position(),
                        doc.is_modified(),
                    ))
                }else{
                    Some(ServerResponse::CursorPosition(doc.get_client_cursor_position(), doc.cursor_position()))
                }
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        }
        ServerAction::MoveCursorDown => {
            if let Some(doc) = editor.document_mut(){
                doc.move_cursor_down();
                let should_update_client_view = doc.scroll_view_following_cursor();
                if should_update_client_view{
                    Some(ServerResponse::DisplayView(
                        doc.get_client_view_text(), 
                        doc.get_client_view_line_numbers(),
                        doc.get_client_cursor_position(), 
                        doc.cursor_position(),
                        doc.is_modified(),
                    ))
                }else{
                    Some(ServerResponse::CursorPosition(doc.get_client_cursor_position(), doc.cursor_position()))
                }
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::MoveCursorUp => {
            if let Some(doc) = editor.document_mut(){
                doc.move_cursor_up();
                let should_update_client_view = doc.scroll_view_following_cursor();
                if should_update_client_view{
                    Some(ServerResponse::DisplayView(
                        doc.get_client_view_text(), 
                        doc.get_client_view_line_numbers(),
                        doc.get_client_cursor_position(), 
                        doc.cursor_position(),
                        doc.is_modified(),
                    ))
                }else{
                    Some(ServerResponse::CursorPosition(doc.get_client_cursor_position(), doc.cursor_position()))
                }
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::MoveCursorRight => {
            if let Some(doc) = editor.document_mut(){
                doc.move_cursor_right();
                let should_update_client_view = doc.scroll_view_following_cursor();
                if should_update_client_view{
                    Some(ServerResponse::DisplayView(
                        doc.get_client_view_text(), 
                        doc.get_client_view_line_numbers(),
                        doc.get_client_cursor_position(), 
                        doc.cursor_position(),
                        doc.is_modified(),
                    ))
                }else{
                    Some(ServerResponse::CursorPosition(doc.get_client_cursor_position(), doc.cursor_position()))
                }
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::MoveCursorLeft => {
            if let Some(doc) = editor.document_mut(){
                doc.move_cursor_left();
                let should_update_client_view = doc.scroll_view_following_cursor();
                if should_update_client_view{
                    Some(ServerResponse::DisplayView(
                        doc.get_client_view_text(), 
                        doc.get_client_view_line_numbers(),
                        doc.get_client_cursor_position(), 
                        doc.cursor_position(),
                        doc.is_modified(),
                    ))
                }else{
                    Some(ServerResponse::CursorPosition(doc.get_client_cursor_position(), doc.cursor_position()))
                }
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::MoveCursorLineEnd => {
            if let Some(doc) = editor.document_mut(){
                doc.move_cursor_end();
                let should_update_client_view = doc.scroll_view_following_cursor();
                if should_update_client_view{
                    Some(ServerResponse::DisplayView(
                        doc.get_client_view_text(), 
                        doc.get_client_view_line_numbers(),
                        doc.get_client_cursor_position(), 
                        doc.cursor_position(),
                        doc.is_modified()
                    ))
                }else{
                    Some(ServerResponse::CursorPosition(doc.get_client_cursor_position(), doc.cursor_position()))
                }
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::MoveCursorLineStart => {
            if let Some(doc) = editor.document_mut(){
                doc.move_cursor_home();
                let should_update_client_view = doc.scroll_view_following_cursor();
                if should_update_client_view{
                    Some(ServerResponse::DisplayView(
                        doc.get_client_view_text(), 
                        doc.get_client_view_line_numbers(),
                        doc.get_client_cursor_position(), 
                        doc.cursor_position(),
                        doc.is_modified(),
                    ))
                }else{
                    Some(ServerResponse::CursorPosition(doc.get_client_cursor_position(), doc.cursor_position()))
                }
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::MoveCursorPageDown => {
            if let Some(doc) = editor.document_mut(){
                doc.move_cursor_page_down();
                let should_update_client_view = doc.scroll_view_following_cursor();
                if should_update_client_view{
                    Some(ServerResponse::DisplayView(
                        doc.get_client_view_text(), 
                        doc.get_client_view_line_numbers(),
                        doc.get_client_cursor_position(), 
                        doc.cursor_position(),
                        doc.is_modified(),
                    ))
                }else{
                    Some(ServerResponse::CursorPosition(doc.get_client_cursor_position(), doc.cursor_position()))
                }
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::MoveCursorPageUp => {
            if let Some(doc) = editor.document_mut(){
                doc.move_cursor_page_up();
                let should_update_client_view = doc.scroll_view_following_cursor();
                if should_update_client_view{
                    Some(ServerResponse::DisplayView(
                        doc.get_client_view_text(), 
                        doc.get_client_view_line_numbers(),
                        doc.get_client_cursor_position(), 
                        doc.cursor_position(),
                        doc.is_modified(),
                    ))
                }else{
                    Some(ServerResponse::CursorPosition(doc.get_client_cursor_position(), doc.cursor_position()))
                }
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::InserChar(c) => {
            if let Some(doc) = editor.document_mut(){
                doc.insert_char(c);
                Some(ServerResponse::DisplayView(
                    doc.get_client_view_text(), 
                    doc.get_client_view_line_numbers(),
                    doc.get_client_cursor_position(), 
                    doc.cursor_position(),
                    doc.is_modified()
                ))
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        }
        ServerAction::InsertNewline => {
            if let Some(doc) = editor.document_mut(){
                doc.insert_newline();
                let _ = doc.scroll_view_following_cursor();
                Some(ServerResponse::DisplayView(
                    doc.get_client_view_text(), 
                    doc.get_client_view_line_numbers(),
                    doc.get_client_cursor_position(), 
                    doc.cursor_position(),
                    doc.is_modified()
                ))
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        }
        ServerAction::InsertTab => {
            if let Some(doc) = editor.document_mut(){
                doc.tab();
                Some(ServerResponse::DisplayView(
                    doc.get_client_view_text(), 
                    doc.get_client_view_line_numbers(),
                    doc.get_client_cursor_position(), 
                    doc.cursor_position(),
                    doc.is_modified()
                ))
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        }
    }
}
