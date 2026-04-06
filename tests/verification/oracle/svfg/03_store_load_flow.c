// PURPOSE: Value flows through store+load pair
void test() {
    int x = 42;
    int *p = &x;
    int y = *p;  // value of x flows through store+load
    (void)y;
}
