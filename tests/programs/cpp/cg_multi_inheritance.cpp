// CG refinement E2E: Multiple inheritance vtable handling.
// CHA resolves e->exec(cmd) through second base class.
#include <cstdlib>
#include <cstdio>

class Logger {
public:
    virtual void log(const char *msg) { puts(msg); }
    virtual ~Logger() = default;
};

class Executor {
public:
    virtual void exec(const char *cmd) = 0;
    virtual ~Executor() = default;
};

class Service : public Logger, public Executor {
public:
    void log(const char *msg) override { puts(msg); }
    void exec(const char *cmd) override { system(cmd); }
};

void run(Executor *e, const char *cmd) {
    e->exec(cmd);  // virtual call via second base
}

int main() {
    const char *input = getenv("CMD");
    Service svc;
    run(&svc, input);
    return 0;
}
