Headers that fill gaps in Tree-sitter's wasm libc headers, for grammars whose scanners use C library
functions Tree-sitter's officially supported grammars don't. They're searched before Tree-sitter's
own headers and pull those in with `#include_next`.

Everything here is `static inline` or a macro, so nothing needs to exist at link time.
