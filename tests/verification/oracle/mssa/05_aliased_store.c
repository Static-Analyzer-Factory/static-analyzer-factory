// PURPOSE: Store through aliased pointer — MSSA must model indirect writes
void test() {
    int x = 0;
    int *p = &x;
    *p = 42;
    int y = x;  // reaching def is *p = 42 (through alias)
    (void)y;
}
