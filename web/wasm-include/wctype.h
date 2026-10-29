#ifndef LUNA_WASM_WCTYPE_H_
#define LUNA_WASM_WCTYPE_H_

// C's <wctype.h> provides wchar_t through <wchar.h>; Tree-sitter's doesn't.
#include <wchar.h>
#include_next <wctype.h>

#endif // LUNA_WASM_WCTYPE_H_
