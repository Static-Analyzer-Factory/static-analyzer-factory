// SVFG E2E test: Diamond CFG with Phi merge.
// if-branch stores val1, else-branch stores val2, load after join.
// Verifies MemPhi + IndirectStore/IndirectLoad edges.

int source(void);
void sink(int);

void test(int cond) {
    int x;
    int *p = &x;

    if (cond) {
        *p = source();  // store tainted value in if-branch
    } else {
        *p = 0;         // store clean value in else-branch
    }

    int val = *p;  // load after join — should see MemPhi
    sink(val);
}
