use std::fs;
use std::result;

type Result<T> = result::Result<T, ()>;

const ADDRESS_OFFSET: u32 = 0x10;

fn print_usage() {
    println!("Usage: hex_editor <file_path>");
}

fn parse_file_path() -> Option<String> {
    let mut args = std::env::args();
    let _ = args.next();

    return args.next();
}

fn print_line(address: &u32, chars: &[char]) {
    print!("{address:08X}\t");

    for ch_to_print in chars.iter() {
        print!("{char:02X} ", char=*ch_to_print as u8);
    }

    let spaces_to_add = (16 - chars.len()) * 3;

    for _i in 0..spaces_to_add {
        print!(" ");
    }
    print!("\t");

    for ch_to_print in chars.iter() {
        if {'!'..'~'}.contains(ch_to_print) {
            print!("{ch_to_print}");
        } else {
            print!(".");
        }
    }
}

fn main() -> Result<()> {
    let file_path = match parse_file_path() {
        Some(file_path) => file_path,
        None => {
            print_usage();
            return Err(());
        }
    };
    
    let data: Vec<_> = fs::read_to_string(&file_path).map_err(|err| {
        eprintln!("Could not read file {file_path}: {err}");
    })?.chars().collect();
   
    let mut current_address = 0x00;
    let mut chars_to_print: Vec<char> = Vec::new();
    chars_to_print.reserve(16);
    for (i, ch) in data.iter().enumerate() {

        if i > 0 && i%16 == 0 {
            print_line(&current_address, &chars_to_print);
            current_address += ADDRESS_OFFSET;

            println!();

            chars_to_print.clear();
        }
        chars_to_print.push(*ch);
    }

    print_line(&current_address, &chars_to_print);

    println!();
    
    Ok(())
}
