/// This will be the main loop for the editor process/daemon(is this really a daemon, by definition?)
use nlo_text_editor_server::{editor::Editor, ServerAction};
use nlo_text_editor_server::ServerResponse;
use nlo_text_editor_server::MESSAGE_SIZE;
use std::{io::Write, net::{TcpListener, TcpStream}};
use std::io::Read;
use std::error::Error;


fn main() -> Result<(), Box<dyn Error>>{
    // set up client/server stuff
    let listener = TcpListener::bind("127.0.0.1:7878").expect("failed to bind to port");
    println!("Server listening on port 7878");
    
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

    Ok(())
}
    
fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn Error>>{
    let mut editor = Editor::default();

    let client_address = stream.peer_addr().unwrap();
    // return connection success response and assign client id?
    
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
                    println!("server emitted: {:#?}", response);
                }
            }
            Err(_) => {
                println!("An error occurred. Terminating connection with {}", client_address);
                //stream.shutdown(std::net::Shutdown::Both).unwrap();
                break;
            }
        }
    }

    Ok(())
}

fn server_action_to_response(action: ServerAction, stream: &mut TcpStream, editor: &mut Editor) -> Option<ServerResponse>{
    match action{
        ServerAction::CloseConnection => {
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            None
        },
        ServerAction::OpenFile(file) => {
            // i think we are trying to open files relative to our current(server) directory. CONFIRMED
            //TODO: open file with absolut paths
            match editor.open_document(&file){
                Ok(_) => {
                    Some(ServerResponse::Acknowledge)
                }
                Err(e) => {
                    Some(ServerResponse::Failed(format!("{}", e)))
                }
            }
        },
        //TODO: Should this return a display view response?
        ServerAction::UpdateClientViewSize(width, height) => {
            if let Some(doc) = editor.document_mut(){
                doc.set_client_view_size(width as usize, height as usize);
                Some(ServerResponse::Acknowledge)
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::RequestClientViewText => {
            match editor.document(){
                Some(doc) => {
                    Some(ServerResponse::DisplayView(doc.get_client_view_text()))
                }
                None => Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::ScrollClientViewDown(amount) => {
            if let Some(doc) = editor.document_mut(){
                doc.scroll_client_view_down(amount);
                Some(ServerResponse::Acknowledge)
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::ScrollClientViewLeft(amount) => {
            if let Some(doc) = editor.document_mut(){
                doc.scroll_client_view_left(amount);
                Some(ServerResponse::Acknowledge)
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::ScrollClientViewRight(amount) => {
            if let Some(doc) = editor.document_mut(){
                doc.scroll_client_view_right(amount);
                Some(ServerResponse::Acknowledge)
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::ScrollClientViewUp(amount) => {
            if let Some(doc) = editor.document_mut(){
                doc.scroll_client_view_up(amount);
                Some(ServerResponse::Acknowledge)
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::RequestClientCursorPosition => {
            if let Some(doc) = editor.document(){
                let client_cursor_position = doc.get_client_cursor_position();
                Some(ServerResponse::DisplayClientCursorPosition(client_cursor_position))
            }else{
                Some(ServerResponse::DisplayClientCursorPosition(None))
            }
        },
        ServerAction::MoveCursorDown => {
            if let Some(doc) = editor.document_mut(){
                doc.move_cursor_down();
                Some(ServerResponse::Acknowledge)
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
        ServerAction::MoveCursorUp => {
            if let Some(doc) = editor.document_mut(){
                let update_client_view = doc.move_cursor_up();
                //TODO: make better response
                Some(ServerResponse::Acknowledge)
            }else{
                Some(ServerResponse::Failed("no document open".to_string()))
            }
        },
    }
}
