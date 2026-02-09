// Simple program for schema discovery
#include <stdlib.h>

int main(int argc, char *argv[]) {
    if (argc < 2) return 1;
    char *cmd = argv[1];
    return system(cmd);
}
