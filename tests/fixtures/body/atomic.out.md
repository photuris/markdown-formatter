This sentence very very very very ends with a code span
`cargo clippy --all-targets` and continues.

Wikilinks such as very very very very very
[[Some Note Name|an alias with spaces]] are never split across lines.

A very long URL
https://example.com/a/very/long/path/that/cannot/be/broken/anywhere/at/all/really/truly
stays whole.

An image
![alt text with several words](https://example.com/a-long-image-path.png) is
never split inside its alt text.

A linked image
[![badge alt text here](https://example.com/badge-image-path.svg)](https://example.com/target)
stays whole too.
