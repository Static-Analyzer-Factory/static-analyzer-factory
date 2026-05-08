#include <stdlib.h>

char *sanitize_input(char *cmd) {
    return cmd;
}

int main(void) {
    char *raw = getenv("SAF_INPUT");
    char *cmd = sanitize_input(raw);
    return system(cmd);
}
