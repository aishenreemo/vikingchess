use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use engine::prelude::MagicTable;
use engine::VikingChessResult;
use engine::prelude::Bitboard;
use engine::prelude::Mask;
use engine::prelude::Square;
use rand::RngCore;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

fn main() -> VikingChessResult<()> {
    let magics_array = MagicTable::from(magic_table());

    let ron_string = ron::to_string(&magics_array)?;
    let file_path = Path::new("./dist/magics.ron");

    if let Some(parent_dir) = file_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }

    let mut file = File::create(file_path)?;
    file.write_all(ron_string.as_bytes())?;

    println!("Successfully generated and saved magic masks to magics.ron");

    Ok(())
}

fn blockers_patterns(mask: Mask) -> Vec<Mask> {
    let indeces: Vec<u128> = (0..128).filter(|&i| (mask.0 >> i) & 1 == 1).collect();
    let indeces_len = indeces.len();
    let patterns_len = 1 << indeces_len;
    let to_mask = |i: u128| (0..indeces_len).fold(Mask(0), |a, b| a | Mask(((i >> b) & 1) << indeces[b]));

    (0..patterns_len).map(to_mask).collect()
}

fn magic_table() -> Vec<(Mask, HashMap<Mask, Mask>)> {
    (0..Bitboard::TOTAL_SQUARES)
        .into_par_iter()
        .map(|i| {
            let shift = MagicTable::SHIFTS[i];
            let square = Square::try_from(i).expect("Unable to convert integer to square.");
            let mask = Bitboard::blockers(square);
            let patterns = blockers_patterns(mask);
            let len = patterns.len();

            let result = magic_entry(shift, square, patterns);
            println!("Success ({}) [{}/{}]: {:?}", len, i, Bitboard::TOTAL_SQUARES, result.0);
            result
        })
        .collect()
}

fn magic_entry(shift: u32, square: Square, patterns: Vec<Mask>) -> (Mask, HashMap<Mask, Mask>) {
    let mut used = HashMap::<Mask, Mask>::new();
    let mut magic: u128 = sparse_random_u128();

    loop {
        let mut success = true;
        for pattern in patterns.iter() {
            let index = Mask(pattern.wrapping_mul(magic) >> (128 - shift));
            let moves = Bitboard::legal_moves(square, *pattern);
            if let Vacant(e) = used.entry(index) {
                e.insert(moves);
            } else if used.get(&index) != Some(&moves) {
                success = false;
                break;
            }
        }

        if success {
            break;
        }

        magic = sparse_random_u128();
        used.clear();
    }

    (Mask(magic), used)
}

pub fn sparse_random_u128() -> u128 {
    let mut rng = rand::rng();
    let num1: u128 = rng.next_u64() as u128 | ((rng.next_u64() as u128) << 64);
    let num2: u128 = rng.next_u64() as u128 | ((rng.next_u64() as u128) << 64);
    let num3: u128 = rng.next_u64() as u128 | ((rng.next_u64() as u128) << 64);

    num1 & num2 & num3
}
