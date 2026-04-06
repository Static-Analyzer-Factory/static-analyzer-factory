// PURPOSE: Copy propagates points-to set from source to destination
void test() {
    int x;
    int *p = &x;
    int *q = p;
    *q = 42;
}
