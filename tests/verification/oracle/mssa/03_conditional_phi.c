// PURPOSE: Conditional stores create a phi node at merge point
int rand_bool(void);

void test() {
    int x;
    if (rand_bool()) {
        x = 1;
    } else {
        x = 2;
    }
    int y = x;  // phi of both stores
    (void)y;
}
