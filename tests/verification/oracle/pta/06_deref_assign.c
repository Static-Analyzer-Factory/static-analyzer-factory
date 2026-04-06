// PURPOSE: Store through pointer — *p = &y changes what locations hold
void test() {
    int x, y;
    int *p = &x;
    int **pp = &p;
    *pp = &y;
    // After store, p now points to y (not x)
}
