// PURPOSE: While loop with break creates extra edge
int rand_bool(void);

void test() {
    int i = 0;
    while (i < 100) {
        if (rand_bool()) break;
        i++;
    }
}
