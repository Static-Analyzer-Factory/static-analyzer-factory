#include <stdlib.h>

int main(void) {
    // comment-only edit: this should not change LLVM IR with debuginfo disabled
    char *cmd = getenv("SAF_INPUT");
    return system(cmd);
}
