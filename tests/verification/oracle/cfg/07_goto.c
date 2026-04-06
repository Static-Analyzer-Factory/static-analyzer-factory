// PURPOSE: Goto creates an explicit edge
void test() {
    int x = 0;
    goto skip;
    x = 1;
skip:
    x = 2;
}
