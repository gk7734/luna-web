#ifndef LUNA_WASM_ASSERT_H_
#define LUNA_WASM_ASSERT_H_

#include_next <assert.h>

#ifndef static_assert
#define static_assert _Static_assert
#endif

#endif // LUNA_WASM_ASSERT_H_
