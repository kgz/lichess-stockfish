use image::ImageReader;

pub fn gen_board(board_bytes: [[u8; 8]; 8], mode: &str, last_moves_x_y_vec: Vec<(f32, f32)>, best_move: String) {
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
		}.to_string()

	}


	let last_moved_img = "square brown dark_png_128px.png";
	let last_moved_img1 = "square brown light_png_128px.png";

	let mut board = image::open("./assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/board.png").unwrap();
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
			last_moved = ImageReader::open(format!("./assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/{}", last_moved_img)).unwrap();
		} else {
			last_moved = ImageReader::open(format!("./assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/{}", last_moved_img1)).unwrap();
		}
		count += 1;
		
		let last_moved = last_moved.decode().unwrap();
		let last_moved = last_moved.to_rgba8();
		for i in 0..last_moved.width() {
			for j in 0..last_moved.height(){
				let pixel = last_moved.get_pixel(i, j);
				let pixel = pixel.0;
				if pixel[3] == 0 {
					continue;
				}
		
				let x = x + i as usize;
				let y = y + j as usize;
				let npixel = new_board.get_pixel_mut(x as u32, y as u32);
				npixel.0 = 
					[
						pixel[0],
						pixel[1],
						pixel[2],
						pixel[3],
					];
			}
		}
		board = image::DynamicImage::ImageRgba8(new_board);

		

	}

	// save new board
	board.save_with_format("nboard.png", image::ImageFormat::Png).unwrap();

	// best move is currently in b8c6 format
	// get the x, y of the best move
	let best_move = best_move.chars().collect::<Vec<char>>();
	let mut x = best_move[0] as u8 - 97;
	let y = best_move[1] as u8 - 49;

	// if mode is 'b' then we need to revese the x
	if mode == "b" {
		x = 7 - x;
		// y = 7 - y;
	}

	let x = x as usize;
	let y = y as usize;

	let x = x * 128;
	let y = y * 128;

	let mut new_board = board.to_rgba8();
	for i in 0..128{
		for j in 0..128{
	
			let x = x + i as usize;
			let y = y + j as usize;
			let npixel = new_board.get_pixel_mut(x as u32, y as u32);
			// make yellow
			npixel.0 = 
				[
					204,
					202,
					62,
					255
				];
			
		}
	}
	board = image::DynamicImage::ImageRgba8(new_board);

	// save new board
	board.save_with_format("nboard3.png", image::ImageFormat::Png).unwrap();

	// next position
	let mut x = best_move[2] as u8 - 97;
	let y = best_move[3] as u8 - 49;

	// if mode is 'b' then we need to revese the x
	if mode == "b" {
		x = 7 - x;
		// y = 7 - y;
	}

	let x = x as usize;
	let y = y as usize;

	let x = x * 128;
	let y = y * 128;

	let mut new_board = board.to_rgba8();

	for i in 0..128{
		for j in 0..128{
	
			let x = x + i as usize;
			let y = y + j as usize;
			let npixel = new_board.get_pixel_mut(x as u32, y as u32);
			// make yellow
			npixel.0 = 
				[
					206,
					214,
					128,
					255
				];
			
		}
	}
	board = image::DynamicImage::ImageRgba8(new_board);

	// save new board
	board.save_with_format("nboard4.png", image::ImageFormat::Png).unwrap();


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
		let mut piece = ImageReader::open(format!("./assets/JohnPablok Cburnett Chess set/PNGs/No shadow/128h/{}", piece_name)).unwrap();
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
				npixel.0 = 
					[
						pixel[0],
						pixel[1],
						pixel[2],
						pixel[3],
					];
			}
		}
	}


	board.save_with_format("board.png", image::ImageFormat::Png).unwrap();
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