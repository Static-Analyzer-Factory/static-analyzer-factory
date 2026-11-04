// termination FALSE: for x < 0 the loop sets x = ~1 = -2 and stays there, so
// the loop-header state (x == -2) recurs and the program never terminates.
extern int __VERIFIER_nondet_int(void);

int main(void) {
    int a = 1;
    int x = __VERIFIER_nondet_int();
    while (x < 0) {
        x = ~a;
    }
    return 0;
}
