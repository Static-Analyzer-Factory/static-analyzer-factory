// PURPOSE: Nested if/else creates multiple branch points
int rand_bool(void);

void test() {
    int x = 0;
    if (rand_bool()) {
        if (rand_bool()) {
            x = 1;
        } else {
            x = 2;
        }
    } else {
        x = 3;
    }
}
