// no-overflow FALSE via a COMPILE-TIME constant overflow clang folds away (no IR
// arithmetic for the sentinel to see). The rank-2 TRUE arm must ABSTAIN (the
// -Winteger-overflow probe fires; the CPAchecker gate would reject anyway); the
// UBSan FALSE path then confirms the overflow. Regression for wrong-TRUE=0.
int main(void) {
    int x = (2147483647 + 1) - 23;
    return x;
}
