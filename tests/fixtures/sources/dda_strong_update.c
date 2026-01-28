// DDA Strong Update Test
// Tests: Singleton store precision with strong updates.
// Expected: DDA should apply strong update and find only the latest value.

#include <stdlib.h>

int main() {
    int x = 1;
    int y = 2;
    int *p = &x;  // p points to x

    // Strong update: p now points to y (previous binding killed)
    p = &y;

    // Load should only see y, not x (due to strong update)
    return *p;  // Should be 2
}
