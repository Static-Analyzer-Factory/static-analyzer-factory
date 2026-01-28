// Taint flow through correlated branches — same flag guards source read
// and sink write. Path-insensitive IFDS reports taint; Z3 should filter
// the infeasible cross-branch flow.
#include <stdlib.h>
#include <string.h>

extern char *getenv(const char *);
extern int system(const char *);

void dispatch(int mode) {
    char *data;
    if (mode == 1) {
        // Tainted source
        data = getenv("USER_INPUT");
    } else {
        data = "safe_default";
    }

    if (mode == 1) {
        // mode == 1: data is tainted, but this is the SAME branch
        // so taint flow from getenv → system is feasible
        system(data);
    }
    // If mode != 1: data = "safe_default" → no taint
    // Cross-branch flow (getenv on mode==1 → system on mode!=1) is infeasible
}

int main() {
    dispatch(1);
    return 0;
}
