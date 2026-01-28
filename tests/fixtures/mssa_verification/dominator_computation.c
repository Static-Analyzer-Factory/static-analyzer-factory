// Phase 2: Dominator computation correctness
// Tests various CFG patterns for correct dominance

extern int condition1(void);
extern int condition2(void);
extern int condition3(void);
extern void sink(int);

// Linear CFG: A → B → C → D
// Expected: A dominates {B,C,D}, B dominates {C,D}, C dominates {D}
void linear_cfg(void) {
    int a = 1;    // Block A
    int b = a + 1; // Block B (may merge with A)
    int c = b + 1; // Block C
    int d = c + 1; // Block D
    sink(d);
}

// Diamond CFG: A → {B, C} → D
// Expected: A dominates {B,C,D}, D dominated by A only (not B or C)
void diamond_cfg(int cond) {
    int x;              // Block A
    if (cond) {         // Branch
        x = 10;         // Block B
    } else {
        x = 20;         // Block C
    }
    // Block D - merge point
    sink(x);  // A dominates this, but neither B nor C does
}

// Nested conditionals (deeper diamond)
void nested_diamond(int c1, int c2) {
    int x;              // Block A
    if (c1) {
        if (c2) {
            x = 1;      // Block B1
        } else {
            x = 2;      // Block B2
        }
    } else {
        x = 3;          // Block C
    }
    // Outer merge
    sink(x);
}

// Multi-way branch (switch simulation)
void multi_way_cfg(int sel) {
    int result;
    if (sel == 0) {
        result = 100;
    } else if (sel == 1) {
        result = 200;
    } else if (sel == 2) {
        result = 300;
    } else {
        result = 400;
    }
    // All branches merge here
    sink(result);
}

// Loop with dominator tree
// Entry → Header → Body → Latch → Header (back edge)
// Header dominates Body and Latch
void loop_dominator(int n) {
    int sum = 0;
    for (int i = 0; i < n; i++) {
        // Loop header dominates this body
        sum += i;
    }
    // Exit dominated by entry
    sink(sum);
}

// Irreducible-like pattern (still reducible in C)
void multiple_entry_simulation(int cond1, int cond2) {
    int x = 0;

    if (cond1) {
        x = 1;
        if (cond2) {
            x = x + 10;
        }
    }

    if (cond2) {
        x = x + 100;
    }

    sink(x);
}
