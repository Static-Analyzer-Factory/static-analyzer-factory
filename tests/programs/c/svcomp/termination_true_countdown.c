// termination TRUE: every run counts x down to 0 (no recurring state); the
// non-termination search must never answer false.
extern int __VERIFIER_nondet_int(void);

int main(void) {
    int x = __VERIFIER_nondet_int();
    while (x > 0) {
        x--;
    }
    return 0;
}
