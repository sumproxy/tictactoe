extern crate piston_window;
extern crate find_folder;

use piston_window::*;
    
mod board;
use board::{Board, Player};

fn main() {
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
