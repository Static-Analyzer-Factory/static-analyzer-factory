// PURPOSE: Direct assignment creates a value-flow edge
void test() {
    int x = 42;
    int y = x;
    int z = y;
    (void)z;
}
