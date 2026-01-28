// CWE-200: Information Leak via Network
// Sensitive data on the stack is sent over a socket without sanitization.
//
// Expected finding: secret buffer -> send() (information leak)
#include <string.h>
#include <sys/socket.h>

void handle_request(int sockfd) {
    char secret[64];
    strcpy(secret, "password=hunter2");      // SOURCE: sensitive data
    char response[128];
    memcpy(response, secret, 64);            // propagates sensitive data
    send(sockfd, response, sizeof(response), 0); // SINK: network send
}

int main(void) {
    handle_request(3);
    return 0;
}
