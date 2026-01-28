// SVFG E2E test: source→store→load→sink flow.
// Tests reachable() query finds the path through memory.

int source(void);
void sink(int);

void test(void) {
    int buf;
    int *ptr = &buf;

    int tainted = source();
    *ptr = tainted;        // store tainted to buf
    int result = *ptr;     // load from buf
    sink(result);          // sink
}
