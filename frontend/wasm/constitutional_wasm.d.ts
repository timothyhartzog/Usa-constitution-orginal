/* tslint:disable */
/* eslint-disable */

export class WasmAnalytics {
    free(): void;
    [Symbol.dispose](): void;
    analyze_author_clause_matrix(): string;
    analyze_author_influence(): string;
    analyze_authors(): string;
    analyze_clause_debate_network(): string;
    analyze_clause_issue_matrix(): string;
    analyze_clauses(top_n: number): string;
    analyze_collections(): string;
    analyze_issues(top_n: number): string;
    analyze_overview(): string;
    analyze_ratification(): string;
    analyze_semantic_similarity(): string;
    analyze_temporal_network(): string;
    analyze_word_frequency(top_n: number): string;
    compare_authors(author1: string, author2: string): string;
    filter_chunks(filters: string): string;
    load_corpus(corpus_json: string): string;
    constructor();
}

export class WasmSearchEngine {
    free(): void;
    [Symbol.dispose](): void;
    get_chunk(chunk_id: string): string;
    get_filters(): string;
    get_stats(): string;
    load_corpus(corpus_json: string): string;
    constructor();
    search(query: string, limit: number): string;
    search_with_filters(query: string, filters: string, limit: number): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmanalytics_free: (a: number, b: number) => void;
    readonly __wbg_wasmsearchengine_free: (a: number, b: number) => void;
    readonly wasmanalytics_analyze_author_clause_matrix: (a: number, b: number) => void;
    readonly wasmanalytics_analyze_author_influence: (a: number, b: number) => void;
    readonly wasmanalytics_analyze_authors: (a: number, b: number) => void;
    readonly wasmanalytics_analyze_clause_debate_network: (a: number, b: number) => void;
    readonly wasmanalytics_analyze_clause_issue_matrix: (a: number, b: number) => void;
    readonly wasmanalytics_analyze_clauses: (a: number, b: number, c: number) => void;
    readonly wasmanalytics_analyze_collections: (a: number, b: number) => void;
    readonly wasmanalytics_analyze_issues: (a: number, b: number, c: number) => void;
    readonly wasmanalytics_analyze_overview: (a: number, b: number) => void;
    readonly wasmanalytics_analyze_ratification: (a: number, b: number) => void;
    readonly wasmanalytics_analyze_semantic_similarity: (a: number, b: number) => void;
    readonly wasmanalytics_analyze_temporal_network: (a: number, b: number) => void;
    readonly wasmanalytics_analyze_word_frequency: (a: number, b: number, c: number) => void;
    readonly wasmanalytics_compare_authors: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
    readonly wasmanalytics_filter_chunks: (a: number, b: number, c: number, d: number) => void;
    readonly wasmanalytics_load_corpus: (a: number, b: number, c: number, d: number) => void;
    readonly wasmanalytics_new: () => number;
    readonly wasmsearchengine_get_chunk: (a: number, b: number, c: number, d: number) => void;
    readonly wasmsearchengine_get_filters: (a: number, b: number) => void;
    readonly wasmsearchengine_get_stats: (a: number, b: number) => void;
    readonly wasmsearchengine_load_corpus: (a: number, b: number, c: number, d: number) => void;
    readonly wasmsearchengine_new: () => number;
    readonly wasmsearchengine_search: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly wasmsearchengine_search_with_filters: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export2: (a: number, b: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
