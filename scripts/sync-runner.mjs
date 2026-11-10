#!/usr/bin/env node
// Builds the small stand-in program for this OS and copies it to
// src-tauri/resources, where the app build picks it up and packs it inside.
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const runners = {
  win32: { dir: "src-win", file: "src-win.exe" },
  linux: { dir: "src-linux", file: "src-linux" },
  darwin: { dir: "src-darwin", file: "src-darwin" },
};

const runner = runners[process.platform];
if (!runner) {
  console.error(`Unsupported platform for runner sync: ${process.platform}`);
  process.exit(1);
}

const result = spawnSync("cargo", ["build", "--release"], { cwd: runner.dir, stdio: "inherit" });
if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

mkdirSync(join("src-tauri", "resources"), { recursive: true });
copyFileSync(join(runner.dir, "target", "release", runner.file), join("src-tauri", "resources", runner.file));
console.log(`Copied ${runner.file} to src-tauri/resources/`);
