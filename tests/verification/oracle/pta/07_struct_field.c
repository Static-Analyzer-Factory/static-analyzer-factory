// PURPOSE: Struct field pointer — p->field points to what was stored
struct Node { int val; int *ptr; };

void test() {
    int x;
    struct Node n;
    n.ptr = &x;
    int *p = n.ptr;
    *p = 42;
}
