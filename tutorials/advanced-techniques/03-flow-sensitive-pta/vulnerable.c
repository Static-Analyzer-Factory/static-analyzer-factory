// vulnerable.c -- Flow-sensitive PTA tutorial: connection pool pointer reuse.
//
// A database "connection" pointer is first assigned to a secret-only
// connection, then reassigned to a public connection.  A flow-insensitive
// analysis (Andersen) conservatively says `conn` may point to either
// connection at the read site, so it reports a false-positive taint flow
// from the secret source to the public sink.
//
// Flow-sensitive PTA tracks points-to sets per program point and
// recognises the strong update at the reassignment — after S2, `conn`
// can only point to `pub_conn`, so the sink only receives public data.
//
// Program points:
//   S1: conn = &secret_conn;   → pts(conn) = {secret_conn}
//   S2: conn = &pub_conn;      → pts(conn) = {pub_conn}  (strong update)
//   L1: result = *conn;        → reads from pub_conn only (flow-sensitive)
//
// Andersen (flow-insensitive): pts(conn) = {secret_conn, pub_conn}
// Flow-sensitive:              pts(conn)@L1 = {pub_conn}
//
// Compile:
//   clang-18 -S -emit-llvm -O0 -o vulnerable.ll vulnerable.c

extern int secret_source(void);
extern int public_source(void);
extern void public_sink(int x);

struct Connection {
    int data;
};

void process(void) {
    struct Connection secret_conn;
    struct Connection pub_conn;
    struct Connection *conn;

    // S1: conn points to secret_conn
    conn = &secret_conn;
    conn->data = secret_source();

    // S2: conn reassigned to pub_conn (strong update kills secret_conn)
    conn = &pub_conn;
    conn->data = public_source();

    // L1: load through conn — only pub_conn is reachable (flow-sensitive)
    int result = conn->data;
    public_sink(result);
}

int main(void) {
    process();
    return 0;
}
