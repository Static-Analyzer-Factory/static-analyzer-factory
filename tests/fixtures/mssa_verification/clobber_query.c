// Phase 4: Clobber Query Precision
// Tests: PTA-disambiguated store/load pairs, aliased vs non-aliased

extern void sink(int);

// No alias: p and q point to different locations
// clobber_for(load *q) should NOT include store to *p
void no_alias_clobber(void) {
    int x, y;
    int* p = &x;
    int* q = &y;

    *p = 10;  // Store to x
    *q = 20;  // Store to y

    int r1 = *p;  // Load from x - clobber is store to *p, not *q
    int r2 = *q;  // Load from y - clobber is store to *q, not *p

    sink(r1);
    sink(r2);
}

// Must alias: both point to same location
// clobber_for(load *q) should include store to *p
void must_alias_clobber(void) {
    int x;
    int* p = &x;
    int* q = &x;  // Same target as p

    *p = 100;  // Store through p
    int result = *q;  // Load through q - should see store to *p as clobber

    sink(result);
}

// May alias through function parameter
void may_alias_param(int* p, int* q) {
    *p = 10;
    *q = 20;  // May or may not alias p

    int r = *p;  // Clobber depends on alias analysis
    sink(r);
}

// Chain of clobbers - find most recent
void clobber_chain(void) {
    int x;
    int* p = &x;

    *p = 1;   // First store
    *p = 2;   // Second store (clobbers first)
    *p = 3;   // Third store (clobbers second)

    int result = *p;  // Clobber should be the third store
    sink(result);
}

// Clobber across branches (phi case)
void clobber_with_phi(int cond) {
    int x;
    int* p = &x;

    *p = 0;  // Initial store

    if (cond) {
        *p = 10;  // Store in true branch
    } else {
        *p = 20;  // Store in false branch
    }

    // Clobber for this load should be the memory phi (not a single store)
    int result = *p;
    sink(result);
}

// Array with potential aliasing
void array_clobber(int i, int j) {
    int arr[10];

    arr[i] = 100;  // Store to arr[i]
    arr[j] = 200;  // Store to arr[j] - may alias arr[i] if i==j

    int r = arr[i];  // May be clobbered by arr[j] store if i==j
    sink(r);
}

// No clobber - load sees LiveOnEntry
void no_clobber_live_on_entry(int* external_ptr) {
    // external_ptr points to caller's memory, not modified here
    int val = *external_ptr;  // Clobber is LiveOnEntry (no local store)
    sink(val);
}
