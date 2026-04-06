// PURPOSE: Loop-carried dependency — phi at loop header
void test() {
    int x = 0;
    for (int i = 0; i < 10; i++) {
        x = x + 1;
    }
    int y = x;
    (void)y;
}
