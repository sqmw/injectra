import { createRequire } from "node:module";
import { homedir } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const root = fileURLToPath(new URL("../", import.meta.url));
export const cache = process.env.INJECTRA_CACHE_DIR ?? resolve(process.env.XDG_CACHE_HOME ?? resolve(homedir(), ".cache"), "injectra");
export const dependencies = resolve(cache, "test-tools");
export const requireTool = createRequire(resolve(dependencies, "package.json"));
export const binary = resolve(process.env.CARGO_TARGET_DIR ?? resolve(cache, "cargo-target"), "debug", process.platform === "win32" ? "injectra.exe" : "injectra");
