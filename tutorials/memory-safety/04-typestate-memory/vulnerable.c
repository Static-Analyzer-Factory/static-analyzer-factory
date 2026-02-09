/**
 * Memory allocation lifecycle violations detected by typestate analysis.
 *
 * Typestate analysis tracks the state of each resource (allocated pointer)
 * through a state machine: Allocated -> Freed -> Error
 *
 * Bugs demonstrated:
 * 1. alloc_then_leak() - malloc without free (leak)
 * 2. alloc_double_free() - free called twice (double-free)
 * 3. alloc_use_after_free() - dereference after free (UAF)
 * 4. alloc_correct() - proper alloc/use/free sequence (no bug)
 */
#include <stdlib.h>
#include <string.h>

/* BUG 1: Memory leak - malloc without free */
void alloc_then_leak(void) {
    char *ptr = (char *)malloc(256);
    if (ptr) {
        strcpy(ptr, "This memory will leak");
    }
    /* Missing: free(ptr) */
}

/* BUG 2: Double-free - free called twice */
void alloc_double_free(void) {
    char *ptr = (char *)malloc(128);
    if (ptr) {
        strcpy(ptr, "Data");
        free(ptr);
        free(ptr);  /* BUG: already freed */
    }
}

/* BUG 3: Use-after-free - dereference after free */
void alloc_use_after_free(void) {
    int *ptr = (int *)malloc(sizeof(int));
    if (ptr) {
        *ptr = 42;
        free(ptr);
        int x = *ptr;  /* BUG: reading freed memory */
        (void)x;
    }
}

/* CORRECT: Proper alloc/use/free lifecycle */
void alloc_correct(void) {
    char *ptr = (char *)malloc(64);
    if (ptr) {
        strcpy(ptr, "Correct usage");
        /* Use the memory... */
        free(ptr);
    }
}

int main(void) {
    alloc_then_leak();
    alloc_double_free();
    alloc_use_after_free();
    alloc_correct();
    return 0;
}
