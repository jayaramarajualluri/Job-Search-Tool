// Typed wrappers over `@tauri-apps/api/core`.invoke.  Each command has a
// thin async helper with concrete input/output types.  Stage 6 fills in the
// full surface; this file only ships the minimum the shell needs today.

import { invoke } from "@tauri-apps/api/core";

export async function ping(): Promise<string> {
  return invoke<string>("ping");
}
