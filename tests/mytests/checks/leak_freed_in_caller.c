/**
 * leak_freed_in_caller.c — Allocator wrapper: freed in caller, no leak.
 *
 * my_alloc() wraps malloc and returns the pointer.
 * The caller (main) frees it.  This is NOT a leak.
 *
 * MustNotReach (without exit-scoping) would false-positive here:
 * BFS from malloc in my_alloc() hits my_alloc()'s return (exit node)
 * before reaching free() in the caller.
 *
 * NeverReachSink correctly follows the Return edge into main(),
 * finds the free() sink, and reports no leak.
 *
 * Expected: 0 memory-leak findings.
 *
 * Compile:
 *   clang-18 -S -emit-llvm -O0 -g leak_freed_in_caller.c -o leak_freed_in_caller.ll
 */

#include <stdlib.h>

void *my_alloc(int n) {
    return malloc(n);
}

int main(void) {
    void *p = my_alloc(10);
    free(p);
    return 0;
}
