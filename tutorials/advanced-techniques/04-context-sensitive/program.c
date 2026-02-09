// Context-sensitive pointer analysis (k-CFA)
//
// This program demonstrates the imprecision of context-insensitive PTA:
// a shared wrapper function `make_pair()` allocates two distinct objects
// depending on which call site invokes it. Without context sensitivity,
// all return values from make_pair() merge into one points-to set.
//
// With k=1 context-sensitive PTA, each call site gets its own context,
// and the analysis can distinguish that `a` and `b` point to different
// allocations (the one inside make_pair called from site 1 vs site 2).
//
// This is a realistic pattern in C codebases — wrapper/factory functions
// that allocate objects on behalf of different callers.

#include <stdlib.h>
#include <stdio.h>

struct Pair {
    int x;
    int y;
};

// Shared factory: allocates a Pair with given values.
// Called from two different sites in main().
struct Pair *make_pair(int x, int y) {
    struct Pair *p = (struct Pair *)malloc(sizeof(struct Pair));
    if (!p) return NULL;
    p->x = x;
    p->y = y;
    return p;
}

// Process function — reads through the pointer.
void print_pair(const struct Pair *pair) {
    if (pair) {
        printf("(%d, %d)\n", pair->x, pair->y);
    }
}

int main(void) {
    // Call site 1: create the "origin" pair
    struct Pair *a = make_pair(0, 0);

    // Call site 2: create the "unit" pair
    struct Pair *b = make_pair(1, 1);

    // With CI PTA: a and b both "may point to" the same abstract object
    //   (the malloc inside make_pair). may_alias(a, b) = May.
    //
    // With k=1 CS-PTA: a points to the malloc from context [site1],
    //   b points to the malloc from context [site2].
    //   may_alias(a, b) = No (different context-qualified locations).

    print_pair(a);
    print_pair(b);

    free(a);
    free(b);
    return 0;
}
