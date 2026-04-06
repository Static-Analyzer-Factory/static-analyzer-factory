// PURPOSE: Value flows through aliased pointers
void test() {
    int x = 42;
    int *p = &x;
    int *q = p;
    int y = *q;  // flows from x through alias chain
    (void)y;
}
