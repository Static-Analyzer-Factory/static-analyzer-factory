// no-overflow TRUE (safe): every reachable signed op stays in range for all inputs
// (the guard bounds x to [1,99] before x+1). UBSan never traps under any mini-fuzz
// constant -> `unknown` (never `true`, never a false alarm).
extern int __VERIFIER_nondet_int(void);
int main(void) {
    int x = __VERIFIER_nondet_int();
    if (x > 0 && x < 100) {
        return x + 1;
    }
    return 0;
}
