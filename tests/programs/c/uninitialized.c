// CWE-457: Use of Uninitialized Variable
// A stack variable is used in a conditional branch without being initialized.
//
// Expected finding: 'status' used before definition on some paths
#include <stdio.h>

int main(int argc, char *argv[]) {
    int status;                              // SOURCE: uninitialized stack variable
    if (argc > 2) {
        status = 0;
    }
    // When argc <= 2, status is never assigned
    if (status == 0) {                       // SINK: use of uninitialized value
        printf("ok\n");
    }
    return 0;
}
