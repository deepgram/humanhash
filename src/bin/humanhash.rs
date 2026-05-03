use humanhash::{DEFAULT_WORDS, HumanizeOptions, MAX_WORDS, humanize_with};
use std::process::ExitCode;

fn usage() -> ExitCode {
    eprintln!(
        "humanhash - deterministic BIP39-encoded fingerprints of digests\n\
         \n\
         Usage:\n\
           humanhash <hash> [--words <n>] [--separator <s>]\n\
         \n\
         Options:\n\
           -w, --words <n>      number of words in the output (1..={MAX_WORDS}, default {DEFAULT_WORDS})\n\
           -s, --separator <s>  joiner between words (default '-')\n\
         \n\
         Input is treated as a hex string. Tolerates a leading 0x or urn:uuid: prefix,\n\
         UUID dashes, and whitespace. Same input always produces the same output.\n\
         \n\
         Examples:\n\
           humanhash ac84a4a\n\
           humanhash 0123456789abcdef0123456789abcdef\n\
           humanhash 550e8400-e29b-41d4-a716-446655440000\n\
           humanhash $(git rev-parse HEAD)\n\
           humanhash --words 3 ac84a4a"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut hash: Option<String> = None;
    let mut separator: String = "-".to_string();
    let mut words: u8 = DEFAULT_WORDS;
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "--separator" | "-s" => {
                let Some(next) = args.get(i + 1) else {
                    return usage();
                };
                separator = next.clone();
                i += 2;
            }
            "--words" | "-w" => {
                let Some(next) = args.get(i + 1) else {
                    return usage();
                };
                let Ok(n) = next.parse::<u8>() else {
                    eprintln!("error: --words expects an integer in 1..={MAX_WORDS}");
                    return ExitCode::from(2);
                };
                words = n;
                i += 2;
            }
            "--help" | "-h" => return usage(),
            _ if a.starts_with('-') && a.len() > 1 => return usage(),
            _ => {
                if hash.is_some() {
                    return usage();
                }
                hash = Some(a.clone());
                i += 1;
            }
        }
    }
    let Some(hash) = hash else { return usage() };
    match humanize_with(
        &hash,
        HumanizeOptions {
            words,
            separator: separator.as_str(),
        },
    ) {
        Ok(s) => {
            println!("{s}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
