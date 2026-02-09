// CWE-78: Command injection for SARIF reporting
#include <stdlib.h>
#include <stdio.h>

void run_command(const char *cmd) {
    printf("Running: %s\n", cmd);
    system(cmd);  // SINK
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Usage: program <command>\n");
        return 1;
    }
    run_command(argv[1]);  // SOURCE: argv -> system
    return 0;
}
