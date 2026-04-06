// PURPOSE: Simple store followed by load — reaching def is the store
void test() {
    int x;
    x = 42;
    int y = x;
    (void)y;
}
