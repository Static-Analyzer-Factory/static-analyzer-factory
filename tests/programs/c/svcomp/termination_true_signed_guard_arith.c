// Termination TRUE through the structural ranking proof: the guards read
// signed `nsw` differences (`x - y > 2`), which are exact in a termination
// task (no undefined behaviour, so they never overflow).
extern int __VERIFIER_nondet_int(void);
int main(void) {
    int x = __VERIFIER_nondet_int();
    int y = __VERIFIER_nondet_int();
    if (x >= 0 && y >= 0) {
        while (x - y > 2 || y - x > 2) {
            if (x < y) {
                x = x + 1;
            } else {
                y = y + 1;
            }
        }
    }
    return 0;
}
