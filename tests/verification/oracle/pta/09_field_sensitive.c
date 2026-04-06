// PURPOSE: Two fields in same struct point to different objects
struct Pair { int *first; int *second; };

void test() {
    int x, y;
    struct Pair p;
    p.first = &x;
    p.second = &y;
    int *a = p.first;
    int *b = p.second;
}
