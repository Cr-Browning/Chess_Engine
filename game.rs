use bitflags::bitflags;
use std::collections::VecDeque;
use crate::utils::*;
type PiecePosition = u64;

pub fn bit_to_position(bit: PiecePosition) -> Result<String, String> {
    if bit == 0 {
        return Err("No piece present!".to_string());
    } else {
        let onebit_index = bit_scan(bit);
        return Ok(index_to_position(onebit_index));
    }
}

pub fn position_to_bit(position: &str) -> Result<PiecePosition, String> {
    if position.len() != 2 {
        return Err(format!("Invalid length: {}, string: '{}'", position.len(), position));
    }

    let bytes = position.as_bytes();
    let byte0 = bytes[0];
    if byte0 < 97 || byte0 >= 97 + 8 {
        return Err(format!("Invalid column character: {}, string: '{}'", byte0 as char, position));
    }

    let column = (byte0 - 97) as u32;

    let byte1 = bytes[1];
    let row;

    match (byte1 as char).to_digit(10) {
        Some(number) => if number < 1 || number > 8 {
            return Err(format!("Invalid row character: {}, string: '{}'", byte1 as char, position));
        } else {
            row = number - 1;
        },
        None => return Err(format!("Invalid row character: {}, string '{}'", byte1 as char, position)),
    }

    let square_number = row * 8 + column;
    let bit = (1 as u64) << square_number;

    Ok(bit)
}

static COL_MAP: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
pub fn index_to_position(index: usize) -> String {
    let column = index % 8;
    let row = index / 8 + 1;
    return format!("{}{}", COL_MAP[column], row);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    White,
    Black
}

#[derive(Debug, PartialEq)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King
}

#[derive(Debug, PartialEq)]
pub struct Piece {
    position: PiecePosition,
    color: Color,
    piece_type: PieceType
}

impl Piece {
    fn to_string(&self) -> String {
        let mut result = match self.piece_type {
            PieceType::Pawn => "p ",
            PieceType::Rook => "r ",
            PieceType::Knight => "n ",
            PieceType::Bishop => "b ",
            PieceType::Queen => "q ",
            PieceType::King => "k ",
        }.to_string();

        if self.color == Color::White {
            result.make_ascii_uppercase();
        }

        result
    }
}   

#[derive(Debug, Copy, Clone)]
pub enum Square {
    Empty,
    Occupied(usize),
}

bitflags! {
    pub struct CastlingRights: u8 {
        const NONE = 0;
        const WHITEKINGSIDE = 1 << 0;
        const WHITEQUEENSIDE = 1 << 1;
        const BLACKKINGSIDE = 1 << 2;
        const BLACKQUEENSIDE = 1 << 3;
        const ALL =
            Self::WHITEKINGSIDE.bits
            | Self::WHITEQUEENSIDE.bits
            | Self::BLACKKINGSIDE.bits
            | Self::BLACKQUEENSIDE.bits;
    }
}

// Game type to own the data
pub struct Game {
    pub pieces: Vec<Piece>,
    pub squares: Vec<Square>,
    pub active_color: Color,
    pub castling_rights: CastlingRights,
    pub en_passant: Option<PiecePosition>,
    pub halfmove_clock: usize,
    pub fullmove_number: usize,
}

impl Game {

    fn push_piece_and_square(&mut self, position: usize, color: Color,
                             piece_type: PieceType, index: &mut usize) {
        self.pieces.push(Piece { position: (1 as u64) << position,
                                 color: color,
                                 piece_type: piece_type });
        self.squares.push(Square::Occupied(*index));
        *index += 1;
    }

    fn push_empty_square(&mut self) {
        self.squares.push(Square::Empty);
    }

    pub fn initialize() -> Game {
        Game::read_FEN("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    pub fn to_string(&self) -> String {
        let mut board = "".to_owned();
        let mut temp = "".to_owned();

        for (i, square) in self.squares.iter().enumerate() {
            match square {
                Square::Empty => temp.push_str(&index_to_position(i)),
                Square::Occupied(idx) => temp.push_str(&self.pieces[*idx].to_string()),
            }

            if (i + 1) % 8 == 0 {
                temp.push_str("\n");
                board.insert_str(0, &temp);
                temp.clear();
            }
        }
        board.insert_str(0, &temp);

        board 
    }


    #[allow(non_snake_case)]
    pub fn read_FEN(fen: &str) -> Game {
        let mut game = Game {
            pieces: vec![],
            squares: vec![],
            active_color: Color::White,
            castling_rights: CastlingRights::ALL,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1};

        let (position, rest) = split_on(fen, ' ');

        let mut deque_squares = VecDeque::new();
        let mut piece_index = 0;
        let mut piece_position = 64;
        
        for row in position.splitn(8, |ch| ch == '/') {
            piece_position -= 8;
            let (pieces, squares) = parse_row(&row, piece_index, piece_position);
            
            for p in pieces {
                game.pieces.push(p);
                piece_index += 1;
            }
            for s in squares {
                deque_squares.push_front(s);
            }
        }

        game.squares = Vec::from(deque_squares);


        let (color_to_move, rest) = split_on(rest, ' ');
        game.active_color = match color_to_move {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Unknown color designator: '{}'", color_to_move),
        };


        let (castling_rights, rest) = split_on(rest, ' ');
        let mut castling = CastlingRights::NONE;
        for ch in castling_rights.chars() {
            match ch {
                'K' => castling |= CastlingRights::WHITEKINGSIDE,
                'Q' => castling |= CastlingRights::WHITEQUEENSIDE,
                'k' => castling |= CastlingRights::BLACKKINGSIDE,
                'q' => castling |= CastlingRights::BLACKQUEENSIDE,
                '-' => (),
                other => panic!("Invalid character in castling rights: '{}'", other),
            }
        }
        game.castling_rights = castling;

        let (en_passant, rest) = split_on(rest, ' ');
        match en_passant {
            "-" => game.en_passant = None,
            s => match position_to_bit(s) {
                Err(msg) => panic!("{}", msg),
                Ok(bit) => game.en_passant = Some(bit),
            }
        };


        let (halfmove_clock, rest) = split_on(rest, ' ');
        match halfmove_clock.parse() {
            Ok(number) => game.halfmove_clock = number,
            Err(_) => panic!("Invalid halfmove: {}", halfmove_clock),
        }

        let (fullmove_number, rest) = split_on(rest, ' ');
        match fullmove_number.parse() {
            Ok(number) => game.fullmove_number = number,
            Err(_) => panic!("Invalid halfmove: {}", fullmove_number),
        }

        game
    }
}

fn parse_row(row: &str, mut piece_index: usize, mut piece_position: usize) -> (Vec<Piece>, VecDeque<Square>) {
    let mut pieces = Vec::new();
    let mut squares = VecDeque::new();

    let mut color;


    macro_rules! add_piece {
        ($piece_type:ident) => {
            {
                let piece = Piece {color: color,
                               position: (1 as u64) << piece_position,
                               piece_type: PieceType::$piece_type};
                let square = Square::Occupied(piece_index);
                pieces.push(piece);
                squares.push_front(square);
                piece_position += 1;
                piece_index += 1;
            }
        };
    }


    for ch in row.chars() {
        let is_upper = ch.is_ascii_uppercase();
        color = if is_upper {Color::White} else {Color::Black};
        match ch.to_ascii_lowercase() {
            'r' => add_piece!(Rook),
            'n' => add_piece!(Knight),
            'b' => add_piece!(Bishop),
            'q' => add_piece!(Queen),
            'k' => add_piece!(King),
            'p' => add_piece!(Pawn),
            num => {
                match num.to_digit(10) {
                    None => panic!("Invalid input: {}", num),
                    Some(number) => for i in 0..number {
                        squares.push_front(Square::Empty);
                        piece_position += 1;
                    }
                }
            }
        }
    }

    (pieces, squares)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_initial_position() -> Game {
        let mut game = Game { pieces: vec![], squares: vec![],
                              active_color: Color::White,
                              castling_rights: CastlingRights::ALL,
                              en_passant: None,
                              halfmove_clock: 0,
                              fullmove_number: 1
        };
        let mut piece_index = 0;

        let color = Color::White;

        game.push_piece_and_square(0, color,
                                   PieceType::Rook, &mut piece_index);
        game.push_piece_and_square(1, color,
                                   PieceType::Knight, &mut piece_index);
        game.push_piece_and_square(2, color,
                                   PieceType::Bishop, &mut piece_index);
        game.push_piece_and_square(3, color,
                                   PieceType::Queen, &mut piece_index);
        game.push_piece_and_square(4, color,
                                   PieceType::King, &mut piece_index);
        game.push_piece_and_square(5, color,
                                   PieceType::Bishop, &mut piece_index);
        game.push_piece_and_square(6, color,
                                   PieceType::Knight, &mut piece_index);
        game.push_piece_and_square(7, color,
                                   PieceType::Rook, &mut piece_index);

        for i in 8..16 {
            game.push_piece_and_square(i, color,
                                       PieceType::Pawn, &mut piece_index);
        }

        for i in 16..48 {
            game.push_empty_square();
        }

        let color = Color::Black;
        for i in 48..56 {
            game.push_piece_and_square(i, color,
                                       PieceType::Pawn, &mut piece_index);
        }        

        let offset = 56;
        game.push_piece_and_square(0 + offset, color,
                                   PieceType::Rook, &mut piece_index);
        game.push_piece_and_square(1 + offset, color,
                                   PieceType::Knight, &mut piece_index);
        game.push_piece_and_square(2 + offset, color,
                                   PieceType::Bishop, &mut piece_index);
        game.push_piece_and_square(3 + offset, color,
                                   PieceType::Queen, &mut piece_index);
        game.push_piece_and_square(4 + offset, color,
                                   PieceType::King, &mut piece_index);
        game.push_piece_and_square(5 + offset, color,
                                   PieceType::Bishop, &mut piece_index);
        game.push_piece_and_square(6 + offset, color,
                                   PieceType::Knight, &mut piece_index);
        game.push_piece_and_square(7 + offset, color,
                                   PieceType::Rook, &mut piece_index);
                
        
        game
    }


    #[test]
    fn read_initial_position() {
        let game = Game::initialize();
        let default = get_initial_position();
        assert_eq!(game.active_color, Color::White);
        assert_eq!(game.castling_rights, CastlingRights::ALL);
        assert_eq!(game.en_passant, None);
        assert_eq!(game.halfmove_clock, 0);
        assert_eq!(game.fullmove_number, 1);
        for i in 0..64 {
            match (game.squares[i], default.squares[i]) {
                (Square::Empty, Square::Empty) => (),
                (Square::Occupied(idx1), Square::Occupied(idx2)) => assert_eq!(game.pieces[idx1], default.pieces[idx2]),
                 _ => panic!("Wrong square at index {}", i),
            }
        }
    }

    #[test]
    fn read_fen_black_active() {
        let game = Game::read_FEN("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2");
        assert_eq!(game.active_color, Color::Black);
    }   

    #[test]
    fn read_fen_no_castling() {
        let game = Game::read_FEN("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - - 1 2");
        assert_eq!(game.castling_rights, CastlingRights::NONE);
    }

    #[test]
    fn read_fen_en_passant_allowed() {
        let en_passant_square = "g7";
        let game = Game::read_FEN(&format!("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq {} 1 2", en_passant_square));
        assert_eq!(game.en_passant, Some(position_to_bit(en_passant_square).unwrap()));
    }

    #[test]
    fn read_fen_moveclocks() {
        let game = Game::read_FEN("rnbqkbnr/pp1ppppp/7P/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b - g7 1 2");
        assert_eq!(game.halfmove_clock, 1);
        assert_eq!(game.fullmove_number, 2);
    }

    #[test]
    fn read_all_possible_castling_rights() {
        let mut rights = "".to_owned(); 
        let right_chars = ["K", "Q", "k", "q"];
        for i in 0..(2^4) {
            let bitflag_rights = CastlingRights::from_bits(i).unwrap();
            for j in 0..4 {
                if (i >> j) & 1 != 0 {
                    rights.push_str(right_chars[j]);
                }
            }
            let fen = format!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w {} - 0 1", rights);
            let game = Game::read_FEN(&fen);
            assert_eq!(game.castling_rights, bitflag_rights, "FEN: {}\n\n i: {}", fen, i);
            rights.clear();
        }
    }
}
