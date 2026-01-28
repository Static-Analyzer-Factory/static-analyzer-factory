// Phase 5: Store-to-Load SVFG Edge Construction
// Tests: IndirectDef edges from stores to loads via clobber analysis

extern void sink(int);

// Simple store-to-load edge
void simple_store_load(void) {
    int x;
    int* p = &x;
    int value = 42;

    *p = value;  // Store: creates IndirectDef edge from 'value' to load result
    int result = *p;  // Load: edge target

    sink(result);
}

// Multiple stores, single load - edge from most recent clobber
void multiple_stores_single_load(void) {
    int x;
    int* p = &x;

    *p = 1;  // Overwritten
    *p = 2;  // Overwritten
    *p = 3;  // This is the clobber for the load

    int result = *p;  // IndirectDef from value '3'
    sink(result);
}

// Single store, multiple loads - edges to each load
void single_store_multiple_loads(void) {
    int x;
    int* p = &x;

    *p = 100;  // Single store

    int r1 = *p;  // IndirectDef edge to r1
    int r2 = *p;  // IndirectDef edge to r2
    int r3 = *p;  // IndirectDef edge to r3

    sink(r1);
    sink(r2);
    sink(r3);
}

// Interleaved stores and loads
void interleaved_store_load(void) {
    int x;
    int* p = &x;

    *p = 10;
    int r1 = *p;  // Edge from 10

    *p = 20;
    int r2 = *p;  // Edge from 20

    *p = 30;
    int r3 = *p;  // Edge from 30

    sink(r1);
    sink(r2);
    sink(r3);
}

// Interprocedural store-load (same function for simplicity)
void store_helper(int* p, int val) {
    *p = val;
}

void load_helper(int* p) {
    int result = *p;
    sink(result);
}

void interproc_store_load(void) {
    int x;
    int* p = &x;

    store_helper(p, 999);  // Store in callee
    load_helper(p);        // Load in another callee
}

// No edge when no alias (separate locations)
void no_edge_different_locations(void) {
    int a, b;
    int* pa = &a;
    int* pb = &b;

    *pa = 111;  // Store to a
    int r = *pb;  // Load from b - no IndirectDef edge from store to a

    sink(r);
}
