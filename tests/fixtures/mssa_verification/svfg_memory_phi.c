// Phase 6: Memory Phi Edge Construction
// Tests: IndirectStore, IndirectLoad, PhiFlow edges

extern int condition(void);
extern void sink(int);

// Simple memory phi with two incoming stores
void simple_mem_phi(int cond) {
    int x;
    int* p = &x;

    if (cond) {
        *p = 10;  // IndirectStore edge to MemPhi
    } else {
        *p = 20;  // IndirectStore edge to MemPhi
    }

    // MemPhi merges stores from both branches
    int result = *p;  // IndirectLoad edge from MemPhi
    sink(result);
}

// Nested phi: outer merge depends on inner merge
void nested_mem_phi(int c1, int c2) {
    int x;
    int* p = &x;

    if (c1) {
        if (c2) {
            *p = 1;  // IndirectStore
        } else {
            *p = 2;  // IndirectStore
        }
        // Inner MemPhi here
    } else {
        *p = 3;  // IndirectStore
    }

    // Outer MemPhi: one operand from inner MemPhi (PhiFlow), one from store
    int result = *p;  // IndirectLoad
    sink(result);
}

// Loop creates phi at header
void loop_mem_phi(int n) {
    int x;
    int* p = &x;

    *p = 0;  // Initial store feeds into loop header phi

    for (int i = 0; i < n; i++) {
        // MemPhi at loop header merges:
        // - Initial store (first iteration)
        // - Loop body store (subsequent iterations)

        int cur = *p;  // IndirectLoad from MemPhi
        *p = cur + 1;  // Store feeds back to MemPhi
    }

    // Final MemPhi after loop
    int result = *p;
    sink(result);
}

// Multiple memory phis in sequence
void sequential_mem_phi(int c1, int c2) {
    int x;
    int* p = &x;

    // First diamond
    if (c1) {
        *p = 100;
    } else {
        *p = 200;
    }
    // MemPhi 1

    int intermediate = *p;  // IndirectLoad from MemPhi 1

    // Second diamond
    if (c2) {
        *p = 300;
    } else {
        *p = 400;
    }
    // MemPhi 2

    int final_val = *p;  // IndirectLoad from MemPhi 2

    sink(intermediate);
    sink(final_val);
}

// Phi with live-on-entry as one operand
void phi_with_live_on_entry(int* external, int cond) {
    // external points to caller memory (live-on-entry)

    int x;
    int* p;

    if (cond) {
        p = external;  // Use external (live-on-entry memory)
    } else {
        p = &x;
        *p = 50;       // Local store
    }

    // MemPhi may have live-on-entry as operand (for external path)
    int result = *p;
    sink(result);
}

// Complex: three-way merge
void three_way_mem_phi(int sel) {
    int x;
    int* p = &x;

    if (sel == 0) {
        *p = 1;
    } else if (sel == 1) {
        *p = 2;
    } else {
        *p = 3;
    }

    // MemPhi merges three stores
    int result = *p;
    sink(result);
}
