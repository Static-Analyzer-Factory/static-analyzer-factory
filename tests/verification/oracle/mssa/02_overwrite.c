// PURPOSE: Second store overwrites first — reaching def is the last store
void test() {
    int x;
    x = 1;
    x = 2;
    int y = x;
    (void)y;
}
