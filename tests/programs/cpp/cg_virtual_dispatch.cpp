// CG refinement E2E: Virtual dispatch resolution via CHA.
// CHA resolves p->process(data) to UnsafeProcessor::process.
#include <cstdlib>

class Processor {
public:
    virtual void process(const char *data) = 0;
    virtual ~Processor() = default;
};

class UnsafeProcessor : public Processor {
public:
    void process(const char *data) override {
        system(data);  // sink
    }
};

void run_processor(Processor *p, const char *data) {
    p->process(data);  // virtual call resolved via CHA
}

int main() {
    const char *input = getenv("CMD");
    UnsafeProcessor proc;
    run_processor(&proc, input);
    return 0;
}
