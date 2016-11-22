extern crate piston_window;
extern crate find_folder;
extern crate websocket;

use piston_window::*;
    
mod board;
use board::{Board, Player};

use std::error::Error;

fn main() {
    use std::thread;
    use std::sync::mpsc::channel;
    use std::io::stdin;

    use websocket::{Message, Sender, Receiver};
    use websocket::message::Type;
    use websocket::client::request::Url;
    use websocket::Client;

    let url = Url::parse("ws://localhost:5000").unwrap();
    let request = Client::connect(url).expect("Couldn't connect to server");

    // Send the request and retrieve a response
    let response = request.send().expect("Failed to retrieve a response");

    // Validate the response
    response.validate().expect("Response invalid");

    let (mut sender, mut receiver) = response.begin().split();

    let (tx, rx) = channel();

    let tx_1 = tx.clone();

    let send_loop = thread::spawn(move || {
	loop {
	    // Send loop
	    let message: Message = match rx.recv() {
		Ok(m) => m,
		Err(e) => {
		    println!("Send Loop: {:?}", e);
		    return;
		}
	    };
	    match message.opcode {
		Type::Close => {
		    let _ = sender.send_message(&message);
		    // If it's a close message, just send it and then return.
		    return;
		},
		_ => (),
	    }
	    // Send the message
	    match sender.send_message(&message) {
		Ok(()) => (),
		Err(e) => {
		    println!("Send Loop: {:?}", e);
		    let _ = sender.send_message(&Message::close());
		    return;
		}
	    }
	}
    });

    let receive_loop = thread::spawn(move || {
	// Receive loop
	for message in receiver.incoming_messages() {
	    let message: Message = match message {
		Ok(m) => m,
		Err(e) => {
		    println!("Receive Loop: {:?}", e);
		    let _ = tx_1.send(Message::close());
		    return;
		}
	    };
	    match message.opcode {
		Type::Close => {
		    // Got a close message, so send a close message and return
		    let _ = tx_1.send(Message::close());
		    return;
		}
		Type::Ping => match tx_1.send(Message::pong(message.payload)) {
		    // Send a pong in response
		    Ok(()) => (),
		    Err(e) => {
			println!("Receive Loop: {:?}", e);
			return;
		    }
		},
		// Say what we received
		_ => {
                    match std::str::from_utf8(&message.payload) {
                        Ok(ref s) => println!("Receive Loop: {}", s),
                        Err(ref e) => println!("Receive Loop: Error: {}", e.description()),
                    }
                }
	    }
	}
    });

    loop {
	let mut input = String::new();

	stdin().read_line(&mut input).unwrap();

	let trimmed = input.trim();

	let message = match trimmed {
	    "/close" => {
		// Close the connection
		let _ = tx.send(Message::close());
		break;
	    }
	    // Send a ping
	    "/ping" => Message::ping(b"PING".to_vec()),
	    // Otherwise, just send text
	    _ => Message::text(trimmed.to_string()),
	};

	match tx.send(message) {
	    Ok(()) => (),
	    Err(e) => {
		println!("Main Loop: {:?}", e);
		break;
	    }
	}
    }

    // We're exiting

    println!("Waiting for child threads to exit");

    let _ = send_loop.join();
    let _ = receive_loop.join();

    println!("Exited");

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
    let mut click = false;
    // gameover flag
    let mut gameover = false;
    // the player starting the game
    let mut player = Player::X;
    // current mouse cursor coordinates
    let mut coords = (0.0, 0.0);

    while let Some(e) = window.next() {

        if let Some(button) = e.release_args() {
            if button == Button::Mouse(MouseButton::Left) && !gameover {
                click = true;
            }
        }

        if let Some(pos) = e.mouse_cursor_args() {
            coords = (pos[0], pos[1]);
        }

        if click {
            let x = match coords.0 as u16 {
                0 ... 99 => 0,
                100 ... 199 => 1,
                200 ... 299 => 2,
                _ => unreachable!()
            };
            let y = match coords.1 as u16 {
                0 ... 99 => 0,
                100 ... 199 => 1,
                200 ... 299 => 2,
                _ => unreachable!()
            };
            if board[y][x] == None {
                board[y][x] = Some(player);
                // We have a winner!
                if let Some(winner) = board.winner((y, x)) {
                    match winner {
                        Player::X => window.set_title("X wins!".to_string()),
                        Player::O => window.set_title("O wins!".to_string()),
                    };
                    gameover = true;
                    continue;
                }
                // Pass the turn to the other player and
                // display an invitation for him to make a move
                player = match player {
                    Player::X => {
                        window.set_title("Your move, O!".to_string());
                        Player::O
                    }
                    Player::O => {
                        window.set_title("Your move, X!".to_string());
                        Player::X
                    }
                }
            }
            click = false;
        }

        // Draw the board
        window.draw_2d(&e, |c, g| {
            clear(white, g);
            for (y, row) in board.into_iter().enumerate() {
                let y = y as f64;
                for (x, cell) in row.into_iter().enumerate() {
                    let x = x as f64;
                    let ct = c.transform.trans(100.0*x, 95.0+100.0*y);
                    match *cell {
                        // Print "O" character on the board
                        Some(Player::O) => text::Text::new_color(black, 150).draw(
                            "o",
                            &mut glyphs,
                            &c.draw_state,
                            ct, g
                        ),
                        // Print "X" character on the board
                        Some(Player::X) => text::Text::new_color(black, 150).draw(
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
}
