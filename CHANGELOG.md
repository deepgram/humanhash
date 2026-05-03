# Changelog

## [0.2.2](https://github.com/deepgram/humanhash/compare/v0.2.1...v0.2.2) (2026-05-03)


### Bug Fixes

* decouple humanhash-tag from build-binaries and drop macos-13 runner ([#5](https://github.com/deepgram/humanhash/issues/5)) ([284ca01](https://github.com/deepgram/humanhash/commit/284ca0163a92a603e3c4f6d47c34aaacda7cffda))

## [0.2.1](https://github.com/deepgram/humanhash/compare/v0.2.0...v0.2.1) (2026-05-03)


### Features

* ship humanhash as a composite github action ([c1ff8e2](https://github.com/deepgram/humanhash/commit/c1ff8e2cee00b3d08ef590f97cac293506950131))
* ship humanhash as a composite github action ([#3](https://github.com/deepgram/humanhash/issues/3)) ([3ee7f1c](https://github.com/deepgram/humanhash/commit/3ee7f1ca9393445e567bee57a7878cdc96c5ae03))

## [0.2.0](https://github.com/deepgram/humanhash/compare/v0.1.0...v0.2.0) (2026-05-03)


### ⚠ BREAKING CHANGES

* HashInput enum removed. humanize / humanize_with signatures changed from HashInput<'_> to &str. humanize_bytes / humanize_bytes_with unchanged in shape, behaviour now lossy. output format is N words with no hex tail (default N=4, max N=5).

### Features

* switch to lossy 4-word fingerprint, drop HashInput enum ([c6e27f4](https://github.com/deepgram/humanhash/commit/c6e27f4c0735459406f2ce4705be846b97deecdb))
