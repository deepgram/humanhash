# humanhash · GitHub Action

Deterministic, BIP39-encoded human-readable fingerprint of a hash, digest, or UUID — as a GitHub Action.

```yaml
- uses: deepgram/humanhash/action@v0.2.0
  id: hh
  with:
    hash: ${{ github.sha }}
- run: echo "Build tag = ${{ steps.hh.outputs.humanhash }}"
# Build tag = obvious-dry-burst-debate
```

Same input always produces the same output. The action downloads a prebuilt binary from the matching `humanhash` GitHub Release, so consumer runners need no Rust toolchain.

## Inputs

| Name | Required | Default | Description |
|---|---|---|---|
| `hash` | yes | — | Hex string to fingerprint. Tolerates `0x` / `urn:uuid:` prefix, UUID dashes, and whitespace. |
| `words` | no | `4` | Word count (`1`..=`5`). 4 words ≈ 17.6 trillion fingerprints. |
| `separator` | no | `-` | String joining the words. |
| `version` | no | shipped version | Override the binary version (without leading `v`). Normally you want the default. |

## Outputs

| Name | Description |
|---|---|
| `humanhash` | The deterministic word fingerprint. |

## Examples

### Friendly name on every CI run

```yaml
- uses: deepgram/humanhash/action@v0.2.0
  id: hh
  with:
    hash: ${{ github.sha }}
- run: |
    echo "::notice::This run = ${{ steps.hh.outputs.humanhash }}"
```

### UUID-named release artifact

```yaml
- uses: deepgram/humanhash/action@v0.2.0
  id: hh
  with:
    hash: ${{ github.run_id }}-${{ github.sha }}
    words: 3
    separator: "_"
- run: tar czf "release-${{ steps.hh.outputs.humanhash }}.tar.gz" dist/
```

### PR comment with a memorable build name

```yaml
- uses: deepgram/humanhash/action@v0.2.0
  id: hh
  with:
    hash: ${{ github.event.pull_request.head.sha }}
- uses: peter-evans/create-or-update-comment@v4
  with:
    issue-number: ${{ github.event.pull_request.number }}
    body: |
      Preview deployed: **${{ steps.hh.outputs.humanhash }}** ·
      `${{ github.event.pull_request.head.sha }}`
```

## Pinning

Two equivalent ways to pin:

```yaml
uses: deepgram/humanhash/action@v0.2.0                       # standard semver
uses: deepgram/humanhash/action@correct-horse-battery-staple # humanhash of the release commit
```

Every release publishes both: a SemVer git tag (`v0.2.0`) **and** an extra annotated git tag whose name is the humanhash of the release commit's SHA. They point at the exact same commit.

The humanhash tag is a fingerprint, **not** a cryptographic anchor. The 44-bit fingerprint space (~17.6 trillion) makes accidental collisions astronomically unlikely but a determined attacker with non-trivial compute could grind out a colliding commit. Treat it as a memorability-and-integrity-signal hybrid, not as a substitute for signed tags or `commit-ish` SHAs. For the strictest supply-chain posture, pin to a full SHA: `uses: deepgram/humanhash/action@<full-sha>`.

## Supported runners

Prebuilt binaries are published for:

- `ubuntu-latest` (linux x64)
- `ubuntu-24.04-arm` and self-hosted linux arm64
- `macos-latest` (macOS arm64)
- `macos-13` (macOS Intel)
- `windows-latest` (windows x64)

## License

[Unlicense](../LICENSE).
