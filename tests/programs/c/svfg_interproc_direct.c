// SVFG E2E test: Interprocedural direct flow.
// Function returns tainted value, caller stores and loads back.
// Tests Return + IndirectDef edge combination.

int source(void);
void sink(int);

int get_tainted(void) {
    return source();
}

void test(void) {
    int x;
    int *p = &x;

    int val = get_tainted();  // Return edge
    *p = val;                 // store
    int loaded = *p;          // load
    sink(loaded);
}
