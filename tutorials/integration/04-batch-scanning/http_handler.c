// Large C program: simplified HTTP request handler
// Multiple vulnerability classes: SQLi, XSS, log injection
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// --- Types ---
struct HttpRequest {
    char method[16];
    char path[256];
    char headers[1024];
    char body[4096];
    char query_params[512];
    char auth_token[128];
};

struct HttpResponse {
    int status_code;
    char content_type[64];
    char body[8192];
};

// --- Logging ---
void log_access(const char *method, const char *path, const char *ip) {
    // SINK: log injection if ip or path contains control characters
    fprintf(stderr, "[ACCESS] %s %s from %s\n", method, path, ip);
}

void log_error(const char *message) {
    fprintf(stderr, "[ERROR] %s\n", message);
}

// --- Authentication ---
int auth_check_token(const char *token) {
    if (!token || strlen(token) == 0) return 0;
    // Simplified: just check length
    return strlen(token) > 8;
}

// --- Database ---
void db_query(char *result, size_t result_size, const char *table, const char *filter) {
    // SINK: SQL injection if filter contains unsanitized user input
    snprintf(result, result_size, "SELECT * FROM %s WHERE %s", table, filter);
    printf("[DB] %s\n", result);
}

void db_insert(const char *table, const char *data) {
    printf("[DB] INSERT INTO %s VALUES (%s)\n", table, data);
}

// --- Response Building ---
void render_html(struct HttpResponse *resp, const char *title, const char *body_content) {
    // SINK: XSS if body_content contains unsanitized user input
    snprintf(resp->body, sizeof(resp->body),
             "<html><head><title>%s</title></head><body>%s</body></html>",
             title, body_content);
    strcpy(resp->content_type, "text/html");
}

void render_json(struct HttpResponse *resp, const char *data) {
    snprintf(resp->body, sizeof(resp->body), "{\"data\": \"%s\"}", data);
    strcpy(resp->content_type, "application/json");
}

void send_response(const struct HttpResponse *resp) {
    printf("HTTP/1.1 %d\n", resp->status_code);
    printf("Content-Type: %s\n\n", resp->content_type);
    printf("%s\n", resp->body);
}

// --- Request Parsing ---
void parse_query_params(const char *path, char *params, size_t params_size) {
    const char *q = strchr(path, '?');
    if (q) {
        strncpy(params, q + 1, params_size - 1);
        params[params_size - 1] = '\0';
    } else {
        params[0] = '\0';
    }
}

void parse_header(const char *headers, const char *key, char *value, size_t value_size) {
    const char *found = strstr(headers, key);
    if (found) {
        found += strlen(key);
        while (*found == ':' || *found == ' ') found++;
        const char *end = strchr(found, '\n');
        size_t len = end ? (size_t)(end - found) : strlen(found);
        if (len >= value_size) len = value_size - 1;
        strncpy(value, found, len);
        value[len] = '\0';
    } else {
        value[0] = '\0';
    }
}

int http_parse_request(struct HttpRequest *req, const char *raw) {
    // SOURCE: raw input from network
    if (!raw || strlen(raw) == 0) return -1;

    // Parse method
    const char *space = strchr(raw, ' ');
    if (!space) return -1;
    size_t method_len = (size_t)(space - raw);
    if (method_len >= sizeof(req->method)) method_len = sizeof(req->method) - 1;
    strncpy(req->method, raw, method_len);
    req->method[method_len] = '\0';

    // Parse path
    const char *path_start = space + 1;
    const char *path_end = strchr(path_start, ' ');
    if (!path_end) path_end = path_start + strlen(path_start);
    size_t path_len = (size_t)(path_end - path_start);
    if (path_len >= sizeof(req->path)) path_len = sizeof(req->path) - 1;
    strncpy(req->path, path_start, path_len);
    req->path[path_len] = '\0';

    // Parse query params
    parse_query_params(req->path, req->query_params, sizeof(req->query_params));

    // Parse headers (simplified)
    const char *header_start = strchr(raw, '\n');
    if (header_start) {
        strncpy(req->headers, header_start + 1, sizeof(req->headers) - 1);
        req->headers[sizeof(req->headers) - 1] = '\0';
    }

    // Parse auth token
    parse_header(req->headers, "Authorization", req->auth_token, sizeof(req->auth_token));

    return 0;
}

// --- Route Handlers ---
void handle_user_search(const struct HttpRequest *req, struct HttpResponse *resp) {
    // SQLi: query params flow to db_query without sanitization
    char result[1024];
    db_query(result, sizeof(result), "users", req->query_params);
    render_json(resp, result);
    resp->status_code = 200;
}

void handle_user_profile(const struct HttpRequest *req, struct HttpResponse *resp) {
    // XSS: query params rendered in HTML without escaping
    render_html(resp, "User Profile", req->query_params);
    resp->status_code = 200;
}

void handle_admin_log(const struct HttpRequest *req, struct HttpResponse *resp) {
    // Log injection: path written to log without sanitization
    log_access(req->method, req->path, "127.0.0.1");
    render_json(resp, "logged");
    resp->status_code = 200;
}

void handle_data_insert(const struct HttpRequest *req, struct HttpResponse *resp) {
    db_insert("data", req->body);
    render_json(resp, "inserted");
    resp->status_code = 201;
}

void handle_not_found(struct HttpResponse *resp) {
    render_html(resp, "Not Found", "The requested page was not found.");
    resp->status_code = 404;
}

// --- Router ---
void router_dispatch(const struct HttpRequest *req, struct HttpResponse *resp) {
    if (strstr(req->path, "/api/users/search")) {
        if (!auth_check_token(req->auth_token)) {
            resp->status_code = 401;
            render_json(resp, "unauthorized");
            return;
        }
        handle_user_search(req, resp);
    } else if (strstr(req->path, "/profile")) {
        handle_user_profile(req, resp);
    } else if (strstr(req->path, "/admin/log")) {
        if (!auth_check_token(req->auth_token)) {
            resp->status_code = 401;
            render_json(resp, "unauthorized");
            return;
        }
        handle_admin_log(req, resp);
    } else if (strstr(req->path, "/api/data")) {
        handle_data_insert(req, resp);
    } else {
        handle_not_found(resp);
    }
}

// --- Main ---
int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Usage: program <raw-request>\n");
        return 1;
    }

    struct HttpRequest req;
    memset(&req, 0, sizeof(req));

    if (http_parse_request(&req, argv[1]) != 0) {
        log_error("Failed to parse request");
        return 1;
    }

    struct HttpResponse resp;
    memset(&resp, 0, sizeof(resp));

    router_dispatch(&req, &resp);
    send_response(&resp);

    return 0;
}
