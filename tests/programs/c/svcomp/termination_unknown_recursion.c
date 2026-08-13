// R7 (plan 201): `fact` is loop-free but SELF-RECURSIVE, so the reachable call
// graph has a cycle. "acyclic call graph" fails => abstain => `unknown`. (This is
// the case a naive loop-only check would wrongly call `true`.)
int fact(int n) {
    if (n <= 1) {
        return 1;
    }
    return n * fact(n - 1);
}

int main(void) {
    return fact(5);
}
