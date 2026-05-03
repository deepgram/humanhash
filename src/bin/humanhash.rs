use humanhash::{HashInput, HumanizeOptions, humanize_with};
use std::process::ExitCode;

fn usage() -> ExitCode {
    eprintln!(
        "humanhash - deterministic BIP39-encoded digests\n\
         \n\
         Usage:\n\
           humanhash <hash> [--separator <s>]\n\
         \n\
         Auto-detected hash families (by normalized hex length):\n\
           git short SHA   ( 7 hex chars)\n\
           MD5             (32 hex chars)\n\
           SHA-1 / git long(40 hex chars)\n\
           SHA-256         (64 hex chars)\n\
           UUID v1-v5      (8-4-4-4-12 form)\n\
         \n\
         Examples:\n\
           humanhash ac84a4a\n\
           humanhash 0123456789abcdef0123456789abcdef\n\
           humanhash 550e8400-e29b-41d4-a716-446655440000\n\
           humanhash $(git rev-parse HEAD)"
    );
    ExitCode::from(2)
}

fn detect<'a>(raw: &'a str) -> Option<HashInput<'a>> {
    let trimmed = raw.trim();
    // UUID is the only shape that legitimately keeps dashes after parsing.
    if looks_like_uuid(trimmed) {
        return Some(HashInput::Uuid(trimmed));
    }
    let body = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    let cleaned: String = body
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    if !cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match cleaned.len() {
        7 => Some(HashInput::GitShort7(Box::leak(cleaned.into_boxed_str()))),
        32 => Some(HashInput::Md5(Box::leak(cleaned.into_boxed_str()))),
        40 => Some(HashInput::Sha1(Box::leak(cleaned.into_boxed_str()))),
        64 => Some(HashInput::Sha256(Box::leak(cleaned.into_boxed_str()))),
        _ => None,
    }
}

fn looks_like_uuid(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (i, b) in bytes.iter().enumerate() {
        let want_dash = matches!(i, 8 | 13 | 18 | 23);
        if want_dash {
            if *b != b'-' {
                return false;
            }
        } else if !(*b as char).is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut hex: Option<String> = None;
    let mut separator: String = "-".to_string();
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
            "--help" | "-h" => return usage(),
            _ if a.starts_with('-') && a.len() > 1 => return usage(),
            _ => {
                if hex.is_some() {
                    return usage();
                }
                hex = Some(a.clone());
                i += 1;
            }
        }
    }
    let Some(hex) = hex else { return usage() };
    let Some(detected) = detect(&hex) else {
        eprintln!(
            "error: input shape not recognized; expected git short (7), MD5 (32), SHA-1 (40), SHA-256 (64), or UUID v1-v5"
        );
        return ExitCode::from(2);
    };
    match humanize_with(
        detected,
        HumanizeOptions {
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
