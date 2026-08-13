// R7 (plan 201): a loop-free, call-free `main` — every execution is finite, so
// the program provably terminates => `true`.
int main(void) {
    int x = 5;
    int y = x * 2 + 3;
    return (y > 100) ? 1 : 0;
}
