# Changelog

## [0.2.0](https://github.com/deepgram/humanhash/compare/v0.1.0...v0.2.0) (2026-05-03)


### ⚠ BREAKING CHANGES

* HashInput enum removed. humanize / humanize_with signatures changed from HashInput<'_> to &str. humanize_bytes / humanize_bytes_with unchanged in shape, behaviour now lossy. output format is N words with no hex tail (default N=4, max N=5).

### Features

* switch to lossy 4-word fingerprint, drop HashInput enum ([c6e27f4](https://github.com/deepgram/humanhash/commit/c6e27f4c0735459406f2ce4705be846b97deecdb))
