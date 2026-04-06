// PURPOSE: Simple if/else creates two successor edges from condition block
int rand_bool(void);

void test() {
    int x = 0;
    if (rand_bool()) {
        x = 1;
    } else {
        x = 2;
    }
    x = x + 1;
}
