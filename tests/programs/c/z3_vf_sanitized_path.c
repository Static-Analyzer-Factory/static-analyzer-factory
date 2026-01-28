// Taint is sanitized on one branch but ValueFlow reports flow on merged
// path. Z3 should filter the infeasible flow.
#include <stdlib.h>
#include <string.h>

extern char *getenv(const char *);
extern int system(const char *);

void handle_request(int trusted) {
    char *input = getenv("REQUEST");
    char buf[256];

    if (trusted) {
        // Sanitized path: copy a safe constant
        strcpy(buf, "echo safe");
    } else {
        // Unsanitized path: raw user input
        strncpy(buf, input, sizeof(buf) - 1);
        buf[255] = '\0';
    }

    // ValueFlow may merge both paths and report taint to system()
    // On the trusted branch, buf is not tainted
    system(buf);
}

int main() {
    handle_request(1);
    return 0;
}
