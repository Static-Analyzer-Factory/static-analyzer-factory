// PURPOSE: Nested struct field access
struct Inner { int *ptr; };
struct Outer { struct Inner inner; };

void test() {
    int x;
    struct Outer o;
    o.inner.ptr = &x;
    int *p = o.inner.ptr;
    *p = 42;
}
