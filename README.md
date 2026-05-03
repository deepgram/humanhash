# humanhash

Deterministic, memorable fingerprints of digests. Same hash always renders the same short word string. Built on the [BIP-0039 English wordlist](https://github.com/bitcoin/bips/blob/master/bip-0039/english.txt) (2048 words, public domain).

```text
$ humanhash ac84a4a
swim-interest-stable-neither

$ humanhash 550e8400-e29b-41d4-a716-446655440000
capable-zebra-print-curve

$ humanhash $(git rev-parse HEAD)
obvious-dry-burst-debate
```

## Why

Long hex digests are forgettable. Telling a coworker "the regression is on `ac84a4a0b348cbdf89c92309d6ee8fd2bc59ced0`" is harder than "it's on `swim-interest-stable-neither`". This crate maps any digest to a deterministic, short, memorable BIP39 word string — same input always renders the same output.

This is a **fingerprint, not an encoding**. The output is lossy — the original digest is not recoverable from the words. The point is something a human can remember and read aloud, not a reversible transform.

## Library usage

```rust
use humanhash::humanize;

let tag = humanize("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")?;
// "obvious-dry-burst-debate"
```

Custom word count or separator:

```rust
use humanhash::{humanize_with, HumanizeOptions};

let tag = humanize_with(
    "0123456789abcdef0123456789abcdef",
    HumanizeOptions { words: 3, separator: " " },
)?;
// "ramp pole believe"
```

A `humanize_bytes` / `humanize_bytes_with` pair is available for callers that already have parsed bytes.

### Input handling

`humanize` takes any `&str` and treats it as hex. The following are stripped before parsing:

- leading `0x` / `0X` prefix
- leading `urn:uuid:` prefix
- any `-` characters (so UUIDs work as-is)
- any whitespace

An odd number of hex characters is padded with a trailing `0` nibble. Empty input or non-hex characters return `HumanhashError`.

### Word count

Default is 4 words. Configurable via `HumanizeOptions::words` in the range `1..=5`. The upper bound is set by the 64-bit accumulator used for the fold (5 × 11 = 55 bits ≤ 64).

| words | bits | unique fingerprints |
|---|---|---|
| 3 | 33 | ~8.6 billion |
| 4 (default) | 44 | ~17.6 trillion |
| 5 | 55 | ~36 quadrillion |

## CLI

```bash
$ humanhash 550e8400-e29b-41d4-a716-446655440000
capable-zebra-print-curve

$ humanhash --words 3 ac84a4a
interest-stable-neither

$ humanhash --separator " " $(git rev-parse HEAD)
obvious dry burst debate
```

## GitHub Action

Same tool, also packaged as a composite action — useful for naming releases, tagging preview deployments, or making CI runs memorable in Slack:

```yaml
- uses: deepgram/humanhash/action@v0.2.0
  id: hh
  with:
    hash: ${{ github.sha }}
- run: echo "Build = ${{ steps.hh.outputs.humanhash }}"
```

Every release publishes the SemVer tag (`v0.2.0`) **and** an annotated git tag whose name is the humanhash of the release commit, so you can also pin like:

```yaml
uses: deepgram/humanhash/action@correct-horse-battery-staple
```

See [`action/README.md`](./action/README.md) for inputs, outputs, and examples.

## How it works

1. Input is normalized (strip `0x`/`urn:uuid:` prefix, drop dashes and whitespace) and parsed as hex bytes.
2. The bytes are folded through a 64-bit FNV-1a hash into a `u64` accumulator.
3. The accumulator is masked to `N × 11` bits, where `N` is the configured word count (default 4).
4. Each 11-bit window indexes the BIP39 wordlist (2048 entries = 2¹¹).
5. The words are joined with the configured separator (default `-`).

FNV-1a is used as a uniform mixer so single-bit input changes avalanche into completely different fingerprints, and short inputs don't degenerate into mostly-constant output.

## License

[Unlicense](./LICENSE) — public domain. The BIP-0039 wordlist embedded in `dicts/bip39.txt` is itself public domain (CC0). See [NOTICE](./NOTICE) for upstream credit.

## Inspired by

- [zacharyvoase/humanhash](https://github.com/zacharyvoase/humanhash) — the original 256-word humanhash. This crate diverges on wordlist (BIP39 / 2048 words instead of 256) but the spirit of "hashes that read like English" is the same.
