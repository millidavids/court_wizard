/* tslint:disable */
/* eslint-disable */

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: (a: number, b: number) => number;
  readonly wasm_bindgen__convert__closures_____invoke__h709be03cf565f2b9: (a: number, b: number, c: number) => void;
  readonly wasm_bindgen__closure__destroy__he331ec59a67943f7: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__ha9cfad8d572b68a8: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__h59f5831a460a4e07: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h24068e80ad76f294: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hb140d6c702255374: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hf910e1c895e72323: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h845266e4bfb6f52c: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hba8b55b205f90eba: (a: number, b: number, c: any, d: any) => void;
  readonly wasm_bindgen__closure__destroy__hc0e7dabe471a3a1b: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h005839839a28202c: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__ha918905c306c599c: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hbb495a8a2c6f24ee: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h547d6e080f62dcb9: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h8d6f8d47c18079d9: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h588b69b53d8b4b4e: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__had2340a315cc3049: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__h6cd1edb040d27839: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h05c5a0a4464ceb89: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hf5567bfcfa872e62: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h1ced12e6304baef9: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h2aada0850f8c42b0: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h35021a000af3d2b5: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h9555c7a5f4b26a22: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h29ce5a345dcac1d3: (a: number, b: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
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
