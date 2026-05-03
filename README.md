# humanhash

Deterministic, human-readable representations of digests. Same hash always renders the same dash-separated word string. Built on the [BIP-0039 English wordlist](https://github.com/bitcoin/bips/blob/master/bip-0039/english.txt) (2048 words, public domain).

```text
$ humanhash ac84a4a
prosper-cement-0a

$ humanhash 550e8400-e29b-41d4-a716-446655440000
fence-injury-ability-share-reduce-tumble-ordinary-silk-green-pretty-abandon-00

$ humanhash $(git rev-parse HEAD)
prosper-cement-choice-grief-million-used-cheese-caught-antenna-resist-physical-pistol-sheriff-trash-10
```

## Why

Long hex digests are forgettable. Telling a coworker "the regression is on `ac84a4a0b348cbdf89c92309d6ee8fd2bc59ced0`" is harder than telling them "it's on `prosper-cement-…-trash-10`". This crate maps a digest to a deterministic, lossless, all-bit-preserving sequence of BIP39 words plus a small hex tail for any leftover bits — so the words are memorable AND the original digest is recoverable.

## Supported input shapes

The API takes typed input via the `HashInput` enum so the caller's intent is explicit:

| variant | hex chars | bits | output shape |
|---|---|---|---|
| `HashInput::GitShort7` | 7 | 28 | 2 words + 2-char hex tail |
| `HashInput::Md5` | 32 | 128 | 11 words + 2-char hex tail |
| `HashInput::Sha1` / `HashInput::GitLong` | 40 | 160 | 14 words + 2-char hex tail |
| `HashInput::Sha256` | 64 | 256 | 23 words + 1-char hex tail |
| `HashInput::Uuid` | 8-4-4-4-12 | 128 | same as MD5 |

`Sha1` and `GitLong` are byte-identical (both 40-hex-char SHA-1) — the two variants exist so the call site reads correctly. Same input into either produces the same output.

## Library usage

```rust
use humanhash::{humanize, HashInput};

let tag = humanize(HashInput::Sha256(
    "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
))?;
// "together-mail-awful-cradle-…-throw-5"
```

Custom separator:

```rust
use humanhash::{humanize_with, HashInput, HumanizeOptions};

let tag = humanize_with(
    HashInput::Md5("0123456789abcdef0123456789abcdef"),
    HumanizeOptions { separator: " " },
)?;
// "the baffling … speck def"
```

Raw byte API (`humanize_bytes` / `humanize_bytes_with`) is available for callers that already have parsed bytes.

## CLI

The crate ships a `humanhash` binary for shell-pipeline use.

```bash
$ humanhash 550e8400-e29b-41d4-a716-446655440000
fence-injury-ability-share-reduce-tumble-ordinary-silk-green-pretty-abandon-00

$ humanhash $(git rev-parse HEAD)
prosper-cement-choice-grief-million-used-cheese-caught-antenna-resist-physical-pistol-sheriff-trash-10
```

Auto-detects input shape from normalized hex length. Pass `--separator <s>` to change the joiner (default `-`).

## How it works

1. Input is validated against the requested `HashInput` variant; non-matching shapes return `HumanhashError`.
2. The bytes are read MSB-first as a stream of 11-bit windows.
3. Each 11-bit window indexes the BIP39 wordlist (2048 entries = 2¹¹).
4. Any input bits that don't fill a complete 11-bit window become a hex tail at the end (2-char tail for MD5/SHA-1, 1-char for SHA-256, etc.).
5. Words and the hex tail are joined with the configured separator.

The mapping is fully deterministic and lossless — the same input always renders the same output, and (in principle) the output is decodable back to the original digest.

## License

[Unlicense](./LICENSE) — public domain. The BIP-0039 wordlist embedded in `dicts/bip39.txt` is itself public domain (CC0). See [NOTICE](./NOTICE) for upstream credit.

## Inspired by

- [zacharyvoase/humanhash](https://github.com/zacharyvoase/humanhash) — the original 256-word humanhash. This crate diverges on wordlist (BIP39 / 2048), lossless encoding (preserves every input bit), and typed input enum, but the spirit of "hashes that read like English" is the same.
