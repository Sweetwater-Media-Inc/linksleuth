import { rmSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const WASM_BINDGEN_VERSION = "0.2.130";
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const appDirectory = resolve(scriptDirectory, "..");
const workspaceRoot = resolve(appDirectory, "../..");
const outputDirectory = resolve(appDirectory, "src/generated");
const wasmArtifact = resolve(
  workspaceRoot,
  "target/wasm32-unknown-unknown/release/linksleuth_wasm.wasm",
);

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: workspaceRoot,
    encoding: "utf8",
    stdio: options.capture ? "pipe" : "inherit",
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    if (options.capture && result.stderr) {
      process.stderr.write(result.stderr);
    }
    process.exit(result.status ?? 1);
  }
  return result.stdout?.trim() ?? "";
}

const installedVersion = run("wasm-bindgen", ["--version"], { capture: true });
if (installedVersion !== `wasm-bindgen ${WASM_BINDGEN_VERSION}`) {
  process.stderr.write(
    `Clue requires wasm-bindgen ${WASM_BINDGEN_VERSION}. Install it with:\n` +
      `cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_VERSION} --locked\n`,
  );
  process.exit(1);
}

run("cargo", [
  "build",
  "-p",
  "linksleuth-wasm",
  "--release",
  "--target",
  "wasm32-unknown-unknown",
]);

rmSync(outputDirectory, { recursive: true, force: true });
mkdirSync(outputDirectory, { recursive: true });
run("wasm-bindgen", [
  wasmArtifact,
  "--out-dir",
  outputDirectory,
  "--target",
  "web",
  "--no-typescript",
]);
