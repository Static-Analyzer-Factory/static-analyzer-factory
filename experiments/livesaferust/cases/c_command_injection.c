#include <stdlib.h>

int main(void) {
    char *cmd = getenv("SAF_INPUT");
    return system(cmd);
}
