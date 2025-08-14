use engine::VikingChessResult;
use engine::prelude::Bitboard;
use engine::prelude::Mask;
use engine::prelude::Square;
use rand::RngCore;
use serde::Serialize;
use std::io::{self, Write};
use std::time::Instant;

fn main() -> VikingChessResult<()> {
    let magic_table = MagicTable::new()?;
    println!("{magic_table:?}");
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
struct MagicTable {
    #[serde(with = "serde_arrays")]
    magics: [u64; Bitboard::TOTAL_SQUARES as usize],
    #[serde(with = "serde_arrays")]
    offsets: [usize; Bitboard::TOTAL_SQUARES as usize],
    #[serde(with = "serde_arrays")]
    shifts: [u8; Bitboard::TOTAL_SQUARES as usize],
    attacks: Vec<Mask>,
}

impl MagicTable {
    pub fn new() -> VikingChessResult<Self> {
        let mut magics = [0u64; Bitboard::TOTAL_SQUARES as usize];
        let mut offsets = [0usize; Bitboard::TOTAL_SQUARES as usize];
        let mut shifts = [0u8; Bitboard::TOTAL_SQUARES as usize];
        let mut attacks = Vec::new();
        let mut current_offset = 0;
        
        let start_time = Instant::now();
        print!("\x1b[?25l"); // Hide cursor
        println!("Generating magic bitboard lookup table for rooks...");

        for i in 0..Bitboard::TOTAL_SQUARES {
            let elapsed = start_time.elapsed();
            let progress = (i as f32 + 1.0) / Bitboard::TOTAL_SQUARES as f32 * 100.0;
            print!("\r\x1b[2KProgress: {:>5.1}% | Square: {:<2} ({}/{}) | Relevant bits: {} | Time: {:.2}s",
                   progress, i, i + 1, Bitboard::TOTAL_SQUARES, 
                   Bitboard::moves(Square::try_from(i)?).count_ones(),
                   elapsed.as_secs_f32());
            io::stdout().flush().unwrap();
            
            let square = Square::try_from(i)?;
            let blocker_mask = Bitboard::moves(square);

            let relevant_bits = blocker_mask.count_ones();
            let size = 1 << relevant_bits;

            let all_blockers = MagicTable::every_blockers(blocker_mask);
            let mut attacks_for_square = vec![Mask(0); size];

            let magic = MagicTable::find_magic_number(square, blocker_mask, &all_blockers, &mut attacks_for_square)?;

            magics[i as usize] = magic;
            offsets[i as usize] = current_offset;
            shifts[i as usize] = 64 - relevant_bits as u8;

            attacks.extend(attacks_for_square);
            current_offset += size;
        }

        println!("\r\x1b[2KGeneration complete! Total time: {:.2}s", start_time.elapsed().as_secs_f32());
        print!("\x1b[?25h");

        Ok(MagicTable {
            magics,
            offsets,
            shifts,
            attacks,
        })
    }

    fn every_blockers(mask: Mask) -> Vec<Mask> {
        let mut indeces = vec![];

        for i in 0..Bitboard::TOTAL_SQUARES {
            if (mask.0 >> i) & 1 == 1 {
                indeces.push(i);
            }
        }

        let patterns_len = 1 << indeces.len();
        let mut out = vec![Mask(0); patterns_len];
        for pi in 0..patterns_len {
            for bi in 0..indeces.len() {
                let bit = (pi >> bi) & 1;
                out[pi as usize].0 |= (bit as u128) << indeces[bi];
            }
        }
        out
    }

    fn find_magic_number(
        square: Square,
        blocker_mask: Mask,
        all_blockers: &[Mask],
        attacks_for_square: &mut [Mask],
    ) -> VikingChessResult<u64> {
        let mut rng = rand::rng();
        let relevant_bits = blocker_mask.count_ones();
        let size = 1 << relevant_bits;

        loop {
            let magic: u64 = rng.next_u64();
            println!("\rTrying {magic}");
            let mut used_indices = vec![false; size];
            let mut unique = true;

            for &blockers in all_blockers.iter() {
                let index = ((blockers.0 * magic as u128) >> (64 - relevant_bits)) as usize;
                let index = index % used_indices.len();

                if used_indices[index] {
                    unique = false;
                    println!("bruh {index}");
                    break;
                }

                used_indices[index] = true;

                let legal_moves = Bitboard::legal_moves(square, blockers);
                attacks_for_square[index] = legal_moves;
            }

            if unique {
                println!("\rMagic accepted! {magic}");
                return Ok(magic);
            }
        }
    }
}
