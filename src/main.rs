extern crate piston_window;
extern crate find_folder;
extern crate websocket;
extern crate rustc_serialize;
extern crate mio;

use piston_window::{PistonWindow, WindowSettings, Glyphs, Button, MouseButton, clear,
                    text, ReleaseEvent, MouseCursorEvent, AdvancedWindow, Transformed,
                    FocusEvent};
    
mod board;
use board::{Board, Player};

use std::error::Error;

use rustc_serialize::json::{self, Json};

#[derive(RustcDecodable, Debug)]
struct Move {
    gameover: String,
    last_move: Vec<i8>,
}

#[derive(PartialEq)]
enum GameOver {
    Draw,
    Won,
    Lost,
    Quit,
}

fn main() {
    use websocket::{Message, Sender, Receiver, WebSocketStream};
    use websocket::message::Type;
    use websocket::client::request::Url;
    use websocket::Client;
    use websocket::result::WebSocketError;

    const USAGE: &'static str = "Usage: \n\tcargo run your_id [rival's id]";

    let player_id = match std::env::args().nth(1) {
        Some(player_id) => player_id,
        None => {
            println!("{}", USAGE);
            return;
        }
    };
    
    let rival_id = std::env::args().nth(2).unwrap_or(String::new());

    let url = Url::parse("wss://tictactoe-serv.herokuapp.com").unwrap();
    let request = Client::connect(url).expect("Couldn't connect to server");
    // Send the request and retrieve a response
    let response = request.send().expect("Failed to retrieve a response");
    // Validate the response
    response.validate().expect("Response invalid");

    let (mut sender, mut receiver) = response.begin().split();

    // Send the message
    let black = [0.0, 0.0, 0.0, 1.0];
    let white = [1.0, 1.0, 1.0, 1.0];
    
    let mut window: PistonWindow =
        WindowSettings::new("Tic Tac Toe!", [300, 300])
        .resizable(false)
        .build()
        .unwrap();

    // find the assets folder
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    
    let font = assets.join("Verdana.ttf");
    let factory = window.factory.clone();

    // get font glyphs
    let mut glyphs = Glyphs::new(font, factory).unwrap();
    let mut board = Board::new();
    // click event flag
    let mut process_click = false;
    // mouse coords update flag
    let mut coords_updated = false;
    // the player starting the game
    let mut player = Player::O;
    let mut rival = Player::X;
    // current mouse cursor coordinates
    let mut coords = (0, 0);
    // websocket data
    let mut data = Move { gameover: String::new(), last_move: Vec::new() };
    // disable clicking events for opponent's turn
    let mut ignore_click_events = true;
    // disable incoming messages
    let mut ignore_incoming_messages = false;
    // gameover flag
    let mut gameover = None;
    // timer
    let mut timer = mio::timer::Timer::default();

    //set websocket receiver state to nonblocking
    match receiver.get_ref().get_ref() {
        &WebSocketStream::Tcp(ref inner) => inner.set_nonblocking(true).unwrap(),
	&WebSocketStream::Ssl(ref inner) => inner.get_ref().set_nonblocking(true).unwrap(),
    };

    sender.send_message(&Message::text(format!("{{\"message\":\"connect\",\"id\":\"{}\",\"rival\":\"{}\"}}", player_id, rival_id)))
        .unwrap_or_else(|e| {
            println!("Error: {:?}", e);
            sender.send_message(&Message::close()).unwrap();
	});

    loop {
        
        if !ignore_incoming_messages {
            let result: Result<Message, _> = receiver.recv_message();
            match result {
                Ok(message) => {
                    match message.opcode {

                        Type::Text => {
                            match std::str::from_utf8(&message.payload) {
                                Ok(ref s) => {
                                    let json = Json::from_str(s).expect("Error parsing JSON");
                                    let obj = json.as_object().unwrap();
                                    if let Some(msg) = obj.get("ok") {
                                        println!("{}", msg);
                                    }
                                    else if let Some(error) = obj.get("error") {
                                        println!("{}", error);
                                    }
                                    else if let Some(_) = obj.get("gameover") {
                                        println!("JSON: {:?}", obj);
                                        data = json::decode(s).unwrap();
                                        ignore_click_events = false;
                                        match &*data.gameover {
                                            "won" =>
                                                gameover = Some(GameOver::Won),
                                            "lost" =>
                                                gameover = Some(GameOver::Lost),
                                            "draw" =>
                                                gameover = Some(GameOver::Draw),
                                            "quit" =>
                                                gameover = Some(GameOver::Quit),
                                            "false" => {
                                                gameover = None;
                                                window.set_title(format!("Your turn, {}", player_id));
                                            }
                                            _ => ()
                                        }
                                        if gameover.is_some() {
                                            ignore_incoming_messages = true;
                                        }
                                    }
                                    else {
                                        println!("Unexpected data: {:?}", obj);
                                    }
                                }
                                Err(ref e) => {
                                    println!("Error: {}", e.description());
                                }
                            }
                        },

                        Type::Close => break,

                        _ => (),
                    }
                },
                
	        Err(e) => {
                    match e {
                        WebSocketError::IoError(_) => (),
                        _ => {
                            println!("Error: {:?}", e);
                            break;
                        }
                    }
	        }
            }
        }

        if let Some(e) = window.next() {

            if !ignore_click_events && gameover.is_none() {

                if let Some(_) = e.focus_args() {
                    process_click = false;
                    ignore_click_events = false;
                    continue;
                }

                if data.last_move.len() == 1 && data.last_move[0] == -1 {
                    player = Player::X;
                    rival = Player::O;
                }
                else if data.gameover != "lost" && data.gameover != "draw" && data.gameover != "quit" {
                    let m = data.last_move.clone();
                    board[m[0] as usize][m[1] as usize] = Some(rival);
                }

                if let Some(button) = e.release_args() {
                    if button == Button::Mouse(MouseButton::Left) {
                        if coords_updated {
                            process_click = true;
                            ignore_click_events = true;
                        }
                    }
                }

                if let Some(pos) = e.mouse_cursor_args() {
                    coords = (pos[0] as i16, pos[1] as i16);
                    coords_updated = true;
                }

                if process_click && coords_updated {
                    if coords == (0, 0) {
                        process_click = false;
                        ignore_click_events = false;
                        continue;
                    }
                    let x = match coords.0 {
                          0 ...  99 => 0,
                        100 ... 199 => 1,
                        200 ... 299 => 2,
                        _ => {
                            process_click = false;
                            ignore_click_events = false;
                            continue;
                        }
                    };
                    let y = match coords.1 {
                          0 ...  99 => 0,
                        100 ... 199 => 1,
                        200 ... 299 => 2,
                        _ => {
                            process_click = false;
                            ignore_click_events = false;
                            continue;
                        }
                    };
                    if board[y][x].is_none() {
                        board[y][x] = Some(player);
                        window.set_title("Waiting for opponent's move".to_string());
                        sender.send_message(&Message::text(format!("{{\"move\":[{},{}]}}", y, x))).unwrap();
                    }
                    else {
                        ignore_click_events = false;
                    }

                    process_click = false;
                    coords_updated = false;
                }
            }

            if gameover.is_some() {
                match gameover {
                    Some(GameOver::Won) =>
                        window.set_title("You won!".to_string()),
                    Some(GameOver::Lost) => {
                        window.set_title("You lost!".to_string());
                        let m = data.last_move.clone();
                        board[m[0] as usize][m[1] as usize] = Some(rival);
                    },
                    Some(GameOver::Draw) => {
                        window.set_title("It's a draw!".to_string());
                        let m = data.last_move.clone();
                        if board[m[0] as usize][m[1] as usize].is_none() {
                            board[m[0] as usize][m[1] as usize] = Some(rival);
                        }
                    },
                    Some(GameOver::Quit) => {
                        window.set_title("Opponent quit".to_string());
                    },
                    None => (),
                }

                let _timeout = timer.set_timeout(std::time::Duration::from_secs(5), ());
                match timer.poll() {
                    Some(()) => {
                        ignore_incoming_messages = false;
                        board = Board::new();
                        if let Some(quit) = gameover {
                            if quit == GameOver::Quit {
                                break;
                            }
                        }
                        gameover = None;
                        timer = mio::timer::Timer::default();
                    },
                    None => (),
                }
            }

            // Draw the board
            window.draw_2d(&e, |c, g| {
                clear(white, g);
                for (y, row) in board.0.iter().enumerate() {
                    let y = y as f64;
                    for (x, cell) in row.iter().enumerate() {
                        let x = x as f64;
                        let ct = c.transform.trans(100.0*x, 95.0+100.0*y);
                        match *cell {
                            // Print "O" character on the board
                            Some(Player::O) =>
                                text::Text::new_color(black, 150).draw(
                                    "o",
                                    &mut glyphs,
                                    &c.draw_state,
                                    ct, g
                                ),
                            // Print "X" character on the board
                            Some(Player::X) =>
                                text::Text::new_color(black, 150).draw(
                                    "x",
                                    &mut glyphs,
                                    &c.draw_state,
                                    ct, g
                                ),
                            None => {}
                        }
                    }
                }
            });
        }            
        else {
            break;
        }
    }
}
