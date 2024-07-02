/// This will be the main loop for the editor process/daemon(is this really a daemon, by definition?)
use nlo_text_editor_server::{editor::Editor, ServerAction};
use nlo_text_editor_server::ServerResponse;
use std::{io::Write, net::{TcpListener, TcpStream}};
use std::io::Read;
use std::error::Error;


const MESSAGE_SIZE: usize = 1024;


fn main() -> Result<(), Box<dyn Error>>{
    // set up client/server stuff
    let listener = TcpListener::bind("127.0.0.1:7878").expect("failed to bind to port");
    //println!("Server listening on port 7878");
    
    for stream in listener.incoming(){
        match stream{
            Ok(stream) => {
                std::thread::spawn(move ||{
                    handle_client(stream);
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
    let _editor = Editor::default();

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
                match server_action_to_response(action, &mut stream){
                    Some(response) => {
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
                    None => {}
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

fn server_action_to_response(action: ServerAction, stream: &mut TcpStream) -> Option<ServerResponse>{
    match action{
        ServerAction::CloseConnection => {
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            None
        },
        ServerAction::OpenFile(_file) => {
            // editor.open_document(file);
            // generate text in view
            Some(ServerResponse::DisplayView("file contents".to_string()))
        }
        _ => None
    }
}
