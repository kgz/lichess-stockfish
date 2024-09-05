use core::str;
use std::{process::exit, sync::Arc};

use image::ImageReader;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::models::error::Error;

#[derive(Debug)]
enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceType {
    fn from_str(s: &str) -> Option<PieceType> {
        match s {
            "king" => Some(PieceType::King),
            "queen" => Some(PieceType::Queen),
            "rook" => Some(PieceType::Rook),
            "bishop" => Some(PieceType::Bishop),
            "knight" => Some(PieceType::Knight),
            "pawn" => Some(PieceType::Pawn),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum Color {
    White,
    Black,
}

impl Color {
    fn from_str(s: &str) -> Option<Color> {
        match s {
            "white" => Some(Color::White),
            "black" => Some(Color::Black),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StockfishResponse {
    success: bool,
    evaluation: Option<f32>,
    mate: Option<i32>,
    bestmove: String,
    continuation: String,
}


pub async fn get_html(url: String) -> Result<String, Error> {
    let res = reqwest::get(url).await;

	if res.is_err() {
		return Err(Error::BasicError(format!("Error fetching html {:?}", res.err())));
	}

	let res = res.unwrap();

	if res.status().as_u16() != 200 {
		return Err(Error::BasicError(format!("Error fetching html {:?}", res.status())));
	}

	let body = res.text().await;

	if body.is_err() {
		return Err(Error::BasicError(format!("Error fetching html {:?}", body.err())));
	}
	

	Ok(body.unwrap())
}

pub async fn get_stock_fish(url: Arc<Mutex<&str>>) -> Result<StockfishResponse, Error> {
    let url = url.lock().await;
    let url = url.to_string();
    let res = reqwest::get(url).await.unwrap();
    let body = res.text().await.unwrap();
    let stockfish: Result<StockfishResponse, serde_json::Error> = serde_json::from_str(&body);
    if stockfish.is_err() {
        println!("Error: {:?} {:?}", body, stockfish.err());
        exit(1); // todo lets not do this
    }
    Ok(stockfish.unwrap())
    // Ok(String::from("test"))
}

pub async fn parse_html<'a>(
    url: Arc<Mutex<&str>>,
    channel: Arc<Mutex<&str>>,
) -> Result<([[u8; 8]; 8], &'a str, Vec<(f32, f32)>), Error> {
    let url = url.lock().await;
    let url = url.to_string();

	let channel = channel.lock().await;
	let channel = channel.to_string();
    let mut board = [[0u8; 8]; 8];
    // set board at 3, 5 to 1
    let html = get_html(url).await?;



    let dom = tl::parse(html.as_str(), tl::ParserOptions::default()).unwrap();
    // get title
    let title = dom.query_selector("title").and_then(|mut iter| iter.next());
    if title.is_none() {
        println!("Title not found");
        exit(1);
    }
    let title = title.unwrap();
    println!(
        "Title: {:?}",
        title.get(dom.parser()).unwrap().inner_text(dom.parser())
    );
    println!("Fetching Piece state for: {:?}", channel);
    let pieces = dom.query_selector("piece");

    if !pieces.is_some() {
        println!("Pieces not found");
        exit(1);
    }

    let orientation_b = dom.get_elements_by_class_name("player").next();

    if orientation_b.is_none() {
		return Err(Error::BasicError("Orientation not found".to_string()));
    }
	let orientation_b = orientation_b.unwrap();
	


    // let orientation_w = dom.get_elements_by_class_name("orientation-black").next();
    // let classes = orientation_b
    //     .get(dom.parser())
    //     .unwrap()
    //     .as_tag()
    //     .unwrap()
    //     .attributes()
    //     .class()
    //     .unwrap();

    // there should be 2
    let last_moves = dom.get_elements_by_class_name("last-move");
    // / vec of x and y

    let mut last_moves_x_y_vec: Vec<(f32, f32)> = vec![];

    for last_move in last_moves {
        let last_move = last_move.get(dom.parser()).unwrap().as_tag().unwrap();
        // as raw tag
        let last_move = last_move.attributes().get("style").unwrap().unwrap();
        let last_move: Vec<&str> = str::from_utf8(last_move.as_bytes())
            .unwrap()
            .split_whitespace()
            .collect();
        // top;left
        let last_move: Vec<_> = last_move[0].split(";").collect();
        let y: Vec<&str> = last_move[0].split(":").collect();
        let y: Vec<&str> = y[1].split("%").collect();
        let y = y[0];
        let y = y.parse::<f32>().unwrap();
        // to u8
        // let y: Vec<u8> = y[0].parse().unwrap();

        let x: Vec<_> = last_move[1].split(";").collect();
        let x: Vec<&str> = x[0].split(":").collect();
        let x: Vec<&str> = x[1].split("%").collect();
        let x = x[0];
        let x = x.parse::<f32>().unwrap();

        // println!("Last move: {:?} {:?}", x, y);
        // println!("Last move: {:?}, {:?}", x, y);
        last_moves_x_y_vec.push((x, y));
    }

	let a = dom.get_elements_by_class_name("player");

	let mut classes = None;
	for i in a {
		// if child a haas href of '/@/{channel}' then we are player
		let child = i.get(dom.parser()).unwrap().as_tag().unwrap().children().all(dom.parser()).first().unwrap();
		let href = child.as_tag().unwrap().attributes().get("href").unwrap().unwrap();
		let href = str::from_utf8(href.as_bytes()).unwrap();
		if href.to_lowercase().contains(channel.to_lowercase().as_str()) {
			println!("Player found at index: {:?}", i);
			classes = Some(i.get(dom.parser()).unwrap().as_tag().unwrap().attributes().class().unwrap());
			break;
		}
	}

	if !classes.is_some() {
		return Err(Error::BasicError("Classes not found".to_string()));
	}

	let classes = classes.unwrap();

	// bug here, player is not alwyas the first in the list.
	let nclasses = classes.as_utf8_str();
	let nclasses = nclasses.split_whitespace().collect::<Vec<&str>>();
	println!("classess: {:?}", nclasses.clone());
	println!("has white: {:?}", nclasses.contains(&"white"));
    let mode = match nclasses.contains(&"white")
    {
        // if white is first then it means our player is black
        true => "w",
        false => "b",
    };
    // println!("Orientation W: {:?}", orientation_w);
    println!(
        "Playing as: {:?}",
        match mode {
            "w" => "White",
            "b" => "Black",
            _ => "Unknown",
        }
    );
    let pieces = pieces.unwrap();

    if pieces.clone().count() == 0 {
		return Err(Error::BasicError("Not in an active game".to_string()));
    }

    for piece in pieces.clone().into_iter() {
        let piece = piece.get(dom.parser()).unwrap().as_tag().unwrap();
        let class = piece.attributes().class().unwrap();

        // split class on whitespace, first is color and second is piece type
        let class: Vec<&str> = str::from_utf8(class.as_bytes())
            .unwrap()
            .split_whitespace()
            .collect();
        let color = Color::from_str(class[0]).unwrap();
        let piece_type = PieceType::from_str(class[1]).unwrap();
        let style = piece.attributes().get("style").unwrap().unwrap();
        // grab everythign inside ""
        let style: Vec<&str> = str::from_utf8(style.as_bytes())
            .unwrap()
            .split("\"")
            .collect();
        let style = style[0];

        // split on ;
        let style: Vec<&str> = style.split(";").collect();
        // top = top:xx.x% and left = left:xx.x%
        // get the value of top and left
        let top: Vec<&str> = style[0].split(":").collect();
        let top = top[1];
        // remove % sign
        let top = top.replace("%", "");
        let top = top.parse::<f32>().unwrap();
        let top = board.len() as f32 * (top / 100.0);

        let left: Vec<&str> = style[1].split(":").collect();
        let left = left[1];
        let left = left.replace("%", "");
        let left = left.parse::<f32>().unwrap();
        let left = board[0].len() as f32 * (left / 100.0);

        // println!("top: {:?}, left: {:?}", top, left);

        let piece_type = match piece_type {
            PieceType::King => 1,
            PieceType::Queen => 2,
            PieceType::Rook => 3,
            PieceType::Bishop => 4,
            PieceType::Knight => 5,
            PieceType::Pawn => 6,
        };

        // replace with color
        let color = match color {
            Color::White => 1,
            Color::Black => 2,
        };

        let code = (color << 3) | piece_type;
        board[top as usize][left as usize] = code;
    }

    println!("\nBoard: ");
    for i in 0..8 {
        for j in 0..8 {
            let piece = board[i][j];
            if piece == 0 {
                print!("{:<2}, ", piece);
                continue;
            }

            let color = piece >> 3;
            let piece_type = piece & 0b111;

            let color = match color {
                1 => "W",
                2 => "B",
                _ => panic!(
                    "Invalid color {:?} at {:?}/{:?}, val {:?}",
                    color, i, j, board[i][j]
                ),
            };

            let piece_type = match piece_type {
                1 => "K",
                2 => "Q",
                3 => "R",
                4 => "B",
                5 => "N",
                6 => "P",
                _ => panic!("Invalid piece type"),
            };

            print!("{:<2}, ", format!("{}{}", color, piece_type));
        }
        println!();
    }

    Ok((board, mode, last_moves_x_y_vec))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetStockFishResponse {
    pub is_black: bool,
    pub evaluation: f32,
    pub mate: Option<i32>,
    pub bestmove: String,
    pub continuation: String,
    pub file: String,
}

pub async fn help<'a>(channel: Arc<Mutex<&'a &str>>) -> Result<GetStockFishResponse, Error> {
    let channel = channel.lock().await;
    let channel = channel.to_string();
    let name = channel.clone();
    let url = format!("https://lichess.org/@/{}/tv", channel.clone());
    let url = Arc::new(Mutex::new(url.as_str()));
    let channel = Arc::new(Mutex::new(channel.as_str()));
    let res = parse_html(url, channel).await?;

    let mut board = res.0;
    let mode = res.1;
    let last_moves_x_y_vec = res.2;
    // set board at 3, 5 to 1

    // since we flip the board for stockfish we need to keep a copy of the original board
    let board_for_image = board.clone();

    // if mode is b then we need to flip the board
    if mode == "b" {
        let mut new_board = [[0u8; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                new_board[i][j] = board[7 - i][7 - j];
            }
        }

        board = new_board;
    }

    println!();
    println!("Calculating FEN and sending to stockfish");
    // to fen

    let fen = encode_to_fen(board, mode);
    println!("FEN: {:?}", fen);

    println!();
    println!("fetching stockfish evaluation...");
    let url = format!("https://stockfish.online/api/s/v2.php?fen={}&depth=15", fen);
    let url = Arc::new(Mutex::new(url.as_str()));
    let stockfish = get_stock_fish(url).await?;

    let bestmove = stockfish.bestmove.replace("bestmove ", "");
    let ponder = bestmove.split_whitespace().nth(2).unwrap_or_else(|| "None");
    let bestmove = bestmove.split_whitespace().nth(0).unwrap_or_else(|| "None");
    let _eval = match stockfish.evaluation {
		Some(eval) => eval,
		None => 0.0,
	};
	let mut chance_to_win;
	if stockfish.evaluation.is_some() {
		chance_to_win = (stockfish.evaluation.unwrap()) / 153.0;
		if mode == "b" {
			chance_to_win = 1.0 - chance_to_win;
		}
		chance_to_win = chance_to_win * 100.0;
	} else {
		chance_to_win = 0.0;
	}

	println!("chance to win: {:?}", chance_to_win);

    println!("Best Move: {:?}", bestmove);
    println!("Ponder: {:?}", ponder);
    println!("Evaluation: {:?}", stockfish.evaluation);
    if chance_to_win > 0.0 {
        println!("You are winning by {:.2}%.", chance_to_win);
    } else if chance_to_win < 40.0 {
        println!("Just quit.");
    } else if chance_to_win < 0.0 {
        println!("You are losing by {:.2}%.", chance_to_win);
    } else {
        println!("You are equal.");
    }
    println!("Mate: {:?}", stockfish.mate);
    println!("Continuation: {:?}", stockfish.continuation);

    let path = gen_board(
        board_for_image,
        mode,
        last_moves_x_y_vec,
        bestmove.to_string(),
        name.to_string(),
    );

    let path = String::from(format!("{}.png", path));

    Ok(GetStockFishResponse {
        is_black: mode == "b",
        evaluation: chance_to_win,
        mate: stockfish.mate,
        bestmove: bestmove.to_string(),
        continuation: stockfish.continuation,
        file: path,
    })
}

pub fn gen_board(
    board_bytes: [[u8; 8]; 8],
    mode: &str,
    last_moves_x_y_vec: Vec<(f32, f32)>,
    best_move: String,
    name: String,
) -> String {
    fn get_piece_path(piece_type: String) -> String {
        match piece_type.as_str() {
            "b" => "b_bishop_png_128px.png",
            "q" => "b_queen_png_128px.png",
            "r" => "b_rook_png_128px.png",
            "n" => "b_knight_png_128px.png",
            "p" => "b_pawn_png_128px.png",
            "k" => "b_king_png_128px.png",
            "B" => "w_bishop_png_128px.png",
            "Q" => "w_queen_png_128px.png",
            "R" => "w_rook_png_128px.png",
            "N" => "w_knight_png_128px.png",
            "P" => "w_pawn_png_128px.png",
            "K" => "w_king_png_128px.png",
            _ => panic!("Invalid piece type {:?}", piece_type),
        }
        .to_string()
    }

    let last_moved_img = "square brown dark_png_128px.png";
    let last_moved_img1 = "square brown light_png_128px.png";


	// get project root
	let cwd = std::env::current_dir().unwrap();
    let mut board =
        image::open(cwd.join("src/assets/board.png")).unwrap_or_else(|_| panic!("Error opening {:?}/assets/board.png", cwd.display()));
	// /mnt/dev/lichess-stockfish/src/assets/board.png

    let mut count = 0;
    for (x, y) in last_moves_x_y_vec {
        // x, y are percentages
        let x = x as f32 / 100.0;
        let y = y as f32 / 100.0;

        let x = (x * 8.0) as u8;
        let y = (y * 8.0) as u8;

        let x = x as usize;
        let y = y as usize;

        let x = x * 128;
        let y = y * 128;

        let mut new_board = board.to_rgba8();
        let last_moved;
        // in theory should never be more then 2
        if count % 2 == 0 {
            // last_moved = ImageReader::open(format!(
            //     "./assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/{}",
            //     last_moved_img
            // ))
            // .unwrap();

			last_moved = ImageReader::open(cwd.join(format!(
				"src/assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/{}",
				last_moved_img
			)))
			.unwrap_or_else(|_| panic!("Error opening {:?}/assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/{}", cwd.display(), last_moved_img));
        } else {
            // last_moved = ImageReader::open(format!(
            //     "./assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/{}",
            //     last_moved_img1
            // ))
            // .unwrap();

			last_moved = ImageReader::open(cwd.join(format!(
				"src/assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/{}",
				last_moved_img1
			)))
			.unwrap_or_else(|_| panic!("Error opening {:?}/assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/{}", cwd.display(), last_moved_img1));
        }
        count += 1;

        let last_moved = last_moved.decode().unwrap();
        let last_moved = last_moved.to_rgba8();
        for i in 0..last_moved.width() {
            for j in 0..last_moved.height() {
                let pixel = last_moved.get_pixel(i, j);
                let pixel = pixel.0;
                if pixel[3] == 0 {
                    continue;
                }

                let x = x + i as usize;
                let y = y + j as usize;
                let npixel = new_board.get_pixel_mut(x as u32, y as u32);
                npixel.0 = [pixel[0], pixel[1], pixel[2], pixel[3]];
            }
        }
        board = image::DynamicImage::ImageRgba8(new_board);
    }

    // save new board
    board
        .save_with_format("nboard.png", image::ImageFormat::Png)
        .unwrap();

    // best move is currently in b8c6 format
    // get the x, y of the best move
    let best_move = best_move.chars().collect::<Vec<char>>();
    let mut x = best_move[0] as u8 - 97;
    let mut y = best_move[1] as u8 - 49;

    // if mode is 'b' then we need to revese the x
    if mode == "b" {
        x = 7 - x;
        // y = 7 - y;
    }  else {
		y = 7 - y;
	}

    let x = x as usize;
    let y = y as usize;

    let x = x * 128;
    let y = y * 128;

    let mut new_board = board.to_rgba8();
    for i in 0..128 {
        for j in 0..128 {
            let x = x + i as usize;
            let y = y + j as usize;
            let npixel = new_board.get_pixel_mut(x as u32, y as u32);
            // make yellow
            npixel.0 = [204, 202, 62, 255];
        }
    }
    board = image::DynamicImage::ImageRgba8(new_board);

    // save new board
    board
        .save_with_format("nboard3.png", image::ImageFormat::Png)
        .unwrap();

    // next position
    let mut x = best_move[2] as u8 - 97;
    let mut y = best_move[3] as u8 - 49;

    // if mode is 'b' then we need to revese the x
    if mode == "b" {
        x = 7 - x;
        // y = 7 - y;
    } else {
		y = 7 - y;
	}

    let x = x as usize;
    let y = y as usize;

    let x = x * 128;
    let y = y * 128;

    let mut new_board = board.to_rgba8();

    for i in 0..128 {
        for j in 0..128 {
            let x = x + i as usize;
            let y = y + j as usize;
            let npixel = new_board.get_pixel_mut(x as u32, y as u32);
            // make yellow
            npixel.0 = [206, 214, 128, 255];
        }
    }
    board = image::DynamicImage::ImageRgba8(new_board);

    // save new board
    board
        .save_with_format("nboard4.png", image::ImageFormat::Png)
        .unwrap();

    // place pieces on the board
    let mut pieces = vec![];

    for i in 0..8 {
        for j in 0..8 {
            let piece = board_bytes[i][j];
            if piece == 0 {
                continue;
            }

            let piece_type = get_piece_type(piece);
            let piece_name = get_piece_path(piece_type.to_string());
            pieces.push((piece_name, j, i));
        }
    }

    let mut board = board.to_rgba8();

    for (piece_name, x, y) in pieces {
		let mut piece = ImageReader::open(cwd.join(format!(
			"src/assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/{}",
			piece_name
		)))
		.unwrap_or_else(|_| panic!("Error opening {:?}/assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/{}", cwd.display(), piece_name));

        piece.set_format(image::ImageFormat::Png);
        let mut piece = piece.decode().unwrap();

        let width = piece.width();

        if width > 128 {
            let ratio = 128.0 / width as f32;
            let height = piece.height() as f32 * ratio;
            piece = piece.resize(128, height as u32, image::imageops::FilterType::Nearest);
        }

        let width = piece.width() as usize;

        let offset = (128 - width) / 2;

        let x = (x * 128) + offset;

        let height = piece.height() as usize;
        let offset = (128 - height) / 2;
        let y = (y * 128) + offset;
        let piece = piece.to_rgba8();

        for i in 0..piece.width() {
            for j in 0..piece.height() {
                let pixel = piece.get_pixel(i, j);
                let pixel = pixel.0;
                if pixel[3] == 0 {
                    continue;
                }

                let x = x + i as usize;
                let y = y + j as usize;
                let npixel = board.get_pixel_mut(x as u32, y as u32);
                npixel.0 = [pixel[0], pixel[1], pixel[2], pixel[3]];
            }
        }
    }

    // random name
    // let name = channel; // uuid::Uuid::new_v4().to_string();
    println!("Saving board to pics/{}.png", name);
    board
        .save_with_format(format!("pics/{}.png", name), image::ImageFormat::Png)
        .unwrap();

    // trim . from start
    // let name = name.trim_start_matches(".").to_string();
    println!("Name: {:?}", name);
    return name;
}

pub fn encode_to_fen(board: [[u8; 8]; 8], mode: &str) -> String {
    let mut fen = String::new();
    for i in 0..8 {
        let mut empty = 0;

        // // if whole row is empty
        // if board[i].iter().all(|&x| x == 0) {
        // 	empty += 8;
        // }

        for j in 0..8 {
            let piece = board[i][j];
            if piece == 0 {
                empty += 1;
                continue;
            }

            if empty > 0 {
                fen.push_str(&empty.to_string());
                empty = 0;
            }

            let piece_type = get_piece_type(piece);
            fen.push_str(&format!("{}", piece_type));
        }

        if empty > 0 {
            fen.push_str(&empty.to_string());
        }

        if i < 7 {
            fen.push('/');
        }
    }

    fen = format!("{}%20{}%20KQkq%20-%200%201", fen, mode);
    let fen_unhtml = fen.replace("%20", " ");

    fen_unhtml
}

pub fn get_color(number: u8) -> String {
    let color = match number {
        1 => "W",
        2 => "B",
        _ => panic!("Invalid color {:?}", number),
    };

    color.to_string()
}

pub fn get_piece_type(number: u8) -> String {
    let color = number >> 3;
    let color = get_color(color);

    let piece_type = number & 0b111;
    let piece_type = match piece_type {
        1 => "K",
        2 => "Q",
        3 => "R",
        4 => "B",
        5 => "N",
        6 => "P",
        _ => panic!("Invalid piece type {:?}", number),
    };

    if color == "B" {
        return piece_type.to_string().to_lowercase();
    }

    piece_type.to_string()
}
