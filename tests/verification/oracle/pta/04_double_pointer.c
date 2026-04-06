// PURPOSE: Double pointer dereference — p -> q -> x
void test() {
    int x;
    int *q = &x;
    int **p = &q;
    **p = 42;
}
