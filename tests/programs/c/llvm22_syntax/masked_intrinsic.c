// LLVM 22 removed the alignment argument from `@llvm.masked.load/store/
// gather/scatter`; alignment now rides on the pointer operand's `align`
// attribute. The LLVM loop vectorizer emits these generic masked intrinsics
// when vectorizing a loop that has a conditional store with a known safe
// vector width.
//
// Compile with `-O2 -mavx2` so the loop vectorizer emits a masked store.

#include <stddef.h>

void conditional_store(float* restrict dst, const int* restrict cond,
                        size_t n) {
    for (size_t i = 0; i < n; ++i) {
        if (cond[i]) {
            dst[i] = 1.0f;
        }
    }
}
