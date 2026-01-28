// IFDS E2E: Multi-hop interprocedural taint (3-level call chain)
//
// Taint flows through a deep call chain:
//   getenv() → step_one() → step_two() → step_three() → system()
//
// Tests that IFDS builds and reuses summary edges across multiple
// function boundaries, not just a single call.

#include <stdlib.h>

char *step_three(char *data) {
    return data;
}

char *step_two(char *data) {
    return step_three(data);
}

char *step_one(char *data) {
    return step_two(data);
}

int main() {
    char *env = getenv("DEEP_CMD");
    char *result = step_one(env);
    system(result);
    return 0;
}
