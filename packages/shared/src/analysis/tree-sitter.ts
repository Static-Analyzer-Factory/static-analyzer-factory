/**
 * Tree-sitter LLVM IR parser.
 *
 * Initializes web-tree-sitter and loads the tree-sitter-llvm grammar
 * to parse LLVM IR source text into a concrete syntax tree (CST).
 */

import { Parser, Language, type Node, type Tree } from 'web-tree-sitter';

let parser: Parser | null = null;

/**
 * Initialize tree-sitter with the LLVM IR grammar.
 * Must be called once before parseLLVMIR.
 */
export async function initParser(): Promise<void> {
  if (parser) return;
  const base = import.meta.env.BASE_URL ?? '/';
  await Parser.init({
    locateFile: (scriptName: string) => `${base}${scriptName}`,
  });
  parser = new Parser();
  const lang = await Language.load(`${base}tree-sitter-llvm.wasm`);
  parser.setLanguage(lang);
}

/**
 * Parse LLVM IR source text into a tree-sitter CST.
 * Throws if the parser has not been initialized.
 */
export function parseLLVMIR(source: string): Tree {
  if (!parser) {
    throw new Error('Parser not initialized. Call initParser() first.');
  }
  const tree = parser.parse(source);
  if (!tree) {
    throw new Error('Parser returned null (language not set or parse aborted).');
  }
  return tree;
}

/** Re-export Node type for consumers. */
export type { Node as SyntaxNode, Tree };
