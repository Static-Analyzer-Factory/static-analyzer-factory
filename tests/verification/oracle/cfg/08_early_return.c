// PURPOSE: Early return creates multiple exit paths
int rand_bool(void);

void test() {
    if (rand_bool()) {
        return;
    }
    // more code
    int x = 42;
    (void)x;
}
