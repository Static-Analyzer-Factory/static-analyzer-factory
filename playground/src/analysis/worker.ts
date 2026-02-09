/**
 * Web Worker that runs the analysis pipeline in a background thread.
 *
 * Pipeline: LLVM IR -> tree-sitter parse -> CST-to-AIR -> saf-wasm analyze
 *
 * Messages sent to the worker:
 *   { type: 'analyze', payload: { llvmIR: string } }
 *
 * Messages sent from the worker:
 *   { type: 'status', status: string }
 *   { type: 'air', air: object }
 *   { type: 'results', results: AnalysisResults }
 *   { type: 'error', error: string }
 */

self.onmessage = async (e: MessageEvent) => {
  const { type, payload } = e.data;

  if (type === 'analyze') {
    const { llvmIR, instSourceLines } = payload as {
      llvmIR: string;
      instSourceLines?: Record<string, number>;
    };
    try {
      self.postMessage({ type: 'status', status: 'parsing' });

      const { initParser, parseLLVMIR } = await import('@saf/web-shared/analysis');
      await initParser();
      const tree = parseLLVMIR(llvmIR);

      self.postMessage({ type: 'status', status: 'converting' });

      const { convertToAIR } = await import('@saf/web-shared/analysis');
      const air = convertToAIR(tree, instSourceLines);

      self.postMessage({ type: 'air', air });
      self.postMessage({ type: 'status', status: 'analyzing' });

      const { initWasm, runAnalysis } = await import('@saf/web-shared/analysis');
      await initWasm();
      const results = runAnalysis(JSON.stringify(air));

      self.postMessage({ type: 'results', results });
      self.postMessage({ type: 'status', status: 'ready' });
    } catch (err) {
      self.postMessage({ type: 'error', error: String(err) });
    }
  }
};
