// Genuine taint confirmed by Z3 — no branch sanitization.
#include <stdlib.h>
#include <string.h>

extern char *getenv(const char *);
extern int system(const char *);

void execute() {
    char *cmd = getenv("USER_CMD");
    char buf[512];
    strncpy(buf, cmd, sizeof(buf) - 1);
    buf[511] = '\0';
    // Direct flow: getenv → strncpy → system — always tainted
    system(buf);
}

int main() {
    execute();
    return 0;
}
