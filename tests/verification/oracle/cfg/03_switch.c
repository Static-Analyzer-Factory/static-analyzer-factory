// PURPOSE: Switch statement with multiple cases
int rand_int(void);

void test() {
    int x = rand_int();
    switch (x) {
        case 0: x = 10; break;
        case 1: x = 20; break;
        case 2: x = 30; break;
        default: x = 40; break;
    }
}
