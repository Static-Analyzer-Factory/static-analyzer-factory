// PURPOSE: Three-level pointer chain a -> b -> c -> x
void test() {
    int x;
    int *c = &x;
    int **b = &c;
    int ***a = &b;
    ***a = 42;
}
