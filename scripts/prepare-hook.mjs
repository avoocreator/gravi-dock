import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const isWin = process.platform === "win32";
const binName = isWin ? "gravi-hook.exe" : "gravi-hook";
const srcBin = join("hook-cli", "target", "release", binName);
const outDir = join("src-tauri", "resources");
const destBin = join(outDir, binName);

execFileSync("cargo", ["build", "--release", "--manifest-path", "hook-cli/Cargo.toml"], {
  stdio: "inherit",
});

if (!existsSync(srcBin)) {
  throw new Error(`Expected hook binary not found at ${srcBin}`);
}

mkdirSync(outDir, { recursive: true });
copyFileSync(srcBin, destBin);
