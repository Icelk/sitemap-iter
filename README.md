# sitemap iterator

Follows the [sitemap protocol](https://sitemaps.org/protocol.html).

This can be used to get the items of a sitemap (e.g. [duckduckgo.com/sitemap.xml](https://duckduckgo.com/sitemap.xml)).

## Other formats

PRs with support for other formats are welcome.

If requested, I might work on them.

## MSRV

Currently, Rust version 1.56 is the minimum version supported by this crate, due to the usage of edition 2021.

If you require this to be pushed back, I'm willing to do that. You'll have to solve CI of several Rust versions (see the [Kvarn CI](https://github.com/Icelk/kvarn/tree/main/.github/workflows/)).
