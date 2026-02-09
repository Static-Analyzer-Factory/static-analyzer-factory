/**
 * HTTP request path parser with buffer overflow bug.
 *
 * Parses a URL path from a raw HTTP request line into a fixed-size buffer.
 * The loop copies characters one by one until '?' (query) or ' ' (HTTP version)
 * is reached, but uses <= instead of < for the bounds check, causing a
 * one-byte overflow when the path is exactly MAX_PATH_LEN characters.
 *
 * Bug: Off-by-one error — loop condition uses <= instead of <, allowing
 *      the index to reach MAX_PATH_LEN (writing past the end of the buffer).
 *
 * CWE-120: Buffer Copy without Checking Size of Input
 */
#include <stdlib.h>
#include <string.h>

#define MAX_PATH_LEN 128

/**
 * Extract the URL path from an HTTP request line.
 *
 * Safe version: respects buffer boundary.
 * Input format: "GET /path/to/resource?query HTTP/1.1"
 */
int parse_path_safe(const char *request, char *path_buf) {
    /* Skip the method (e.g., "GET ") */
    const char *p = request;
    while (*p && *p != ' ') p++;
    if (*p == ' ') p++;

    /* Copy the path into the fixed-size buffer */
    int i = 0;
    while (*p && *p != '?' && *p != ' ' && i < MAX_PATH_LEN - 1) {
        path_buf[i] = *p;
        i++;
        p++;
    }
    path_buf[i] = '\0';
    return i;
}

/**
 * Extract the URL path from an HTTP request line.
 *
 * BUGGY version: off-by-one allows writing past buffer end.
 * The condition `i <= MAX_PATH_LEN` should be `i < MAX_PATH_LEN - 1`.
 */
int parse_path_overflow(const char *request, char *path_buf) {
    const char *p = request;
    while (*p && *p != ' ') p++;
    if (*p == ' ') p++;

    /* BUG: Off-by-one — allows i == MAX_PATH_LEN, writing past buffer end */
    int i = 0;
    while (*p && *p != '?' && *p != ' ' && i <= MAX_PATH_LEN) {
        path_buf[i] = *p;
        i++;
        p++;
    }
    path_buf[i] = '\0';  /* This write can be 1 byte past the buffer end */
    return i;
}

/**
 * Process an HTTP request by extracting its path.
 */
int process_request(const char *request) {
    char path[MAX_PATH_LEN];
    int len;

    /* This call is safe — bounded loop */
    len = parse_path_safe(request, path);
    if (len == 0) return -1;

    /* This call has a buffer overflow — off-by-one */
    char path2[MAX_PATH_LEN];
    len = parse_path_overflow(request, path2);

    return len;
}

int main(void) {
    const char *request = "GET /api/v2/users/profile HTTP/1.1";
    return process_request(request);
}
