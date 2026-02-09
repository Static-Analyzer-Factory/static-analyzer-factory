#include <stdlib.h>

struct Point {
    int x;
    int y;
};

int main() {
    struct Point *p = malloc(sizeof(struct Point));
    p->x = 10;
    p->y = 20;

    struct Point *q = p;  // q aliases p
    q->x = 30;

    // Both p->x and q->x are now 30
    int val = p->x + q->y;
    return val;
}
