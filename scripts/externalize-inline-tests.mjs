#!/usr/bin/env node

// Move inline `#[cfg(test)] mod name { ... }` blocks out of Rust sources and
// into sibling `*_tests.rs` files, leaving the repo's out-of-line form behind:
//
//     #[cfg(test)]
//     #[path = "foo_tests.rs"]
//     mod tests;
//
// The module stays a child of the same parent, so `use super::*;`, privacy and
// every relative path inside the body keep working unchanged. Run it against
// the core crate or any vendored submodule checkout:
//
//     node scripts/externalize-inline-tests.mjs <root>... [--write] [--no-fmt]
//
// Without `--write` it only reports. It exits 1 while any inline test module
// remains, including the ones it refuses to move and lists as "manual".

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SKIPPED_DIRS = new Set(["tests", "benches", "examples", "target", "docs"]);
const TEST_FILE_NAMES = new Set(["tests.rs", "test.rs"]);
const CFG_TEST_ATTR = /^#\[cfg\(.*\btest\b.*\)\]$/;
const CFG_NOT_TEST = /\bnot\(\s*test\s*\)/;
const MOD_OPEN = /^([ \t]*)((?:pub(?:\([^)]*\))?[ \t]+)?mod[ \t]+(\w+))[ \t]*\{/gm;
const RAW_STRING = /b?r(#*)"/y;
const IDENT_CHAR = /[A-Za-z0-9_]/;

/** One byte per source character: 1 where it is code, 0 inside a string, char literal or comment. */
export function codeMask(src) {
  const n = src.length;
  const code = new Uint8Array(n).fill(1);
  let i = 0;
  while (i < n) {
    const c = src[i];
    if (c === "/" && src[i + 1] === "/") {
      let j = src.indexOf("\n", i);
      if (j < 0) j = n;
      code.fill(0, i, j);
      i = j;
    } else if (c === "/" && src[i + 1] === "*") {
      let depth = 1;
      let j = i + 2;
      while (j < n && depth > 0) {
        if (src.startsWith("/*", j)) {
          depth += 1;
          j += 2;
        } else if (src.startsWith("*/", j)) {
          depth -= 1;
          j += 2;
        } else {
          j += 1;
        }
      }
      code.fill(0, i, j);
      i = j;
    } else if ((c === "r" || c === "b") && (i === 0 || !IDENT_CHAR.test(src[i - 1])) && rawStringAt(src, i)) {
      const { end } = rawStringAt(src, i);
      code.fill(0, i, end);
      i = end;
    } else if (c === '"') {
      let j = i + 1;
      while (j < n && src[j] !== '"') j += src[j] === "\\" ? 2 : 1;
      j = Math.min(j + 1, n);
      code.fill(0, i, j);
      i = j;
    } else if (c === "'") {
      const end = charLiteralEnd(src, i);
      if (end > 0) code.fill(0, i, end);
      i = end > 0 ? end : i + 1;
    } else {
      i += 1;
    }
  }
  return code;
}

function rawStringAt(src, i) {
  RAW_STRING.lastIndex = i;
  const m = RAW_STRING.exec(src);
  if (!m) return null;
  const closer = `"${m[1]}`;
  const found = src.indexOf(closer, i + m[0].length);
  return { end: found < 0 ? src.length : found + closer.length };
}

/** End index of a char literal starting at `i`, or -1 when the quote opens a lifetime or label. */
function charLiteralEnd(src, i) {
  if (src[i + 1] === "\\") {
    const close = src.indexOf("'", i + 3);
    return close < 0 ? -1 : close + 1;
  }
  const width = src.codePointAt(i + 1) > 0xffff ? 2 : 1;
  return src[i + 1 + width] === "'" && src[i + 1] !== "'" ? i + 2 + width : -1;
}

/** `src` with strings, char literals and comments blanked to spaces, newlines kept. */
function skeleton(src, code) {
  const parts = [];
  let i = 0;
  while (i < src.length) {
    const keep = code[i];
    let j = i;
    while (j < src.length && code[j] === keep) j += 1;
    parts.push(keep ? src.slice(i, j) : src.slice(i, j).replace(/[^\n]/g, " "));
    i = j;
  }
  return parts.join("");
}

function matchBrace(skel, open) {
  let depth = 0;
  for (let i = open; i < skel.length; i += 1) {
    if (skel[i] === "{") depth += 1;
    else if (skel[i] === "}" && --depth === 0) return i;
  }
  return -1;
}

function testFileName(stem, modName) {
  if (modName === "tests") return `${stem}_tests.rs`;
  if (modName.endsWith("_tests")) return `${stem}_${modName}.rs`;
  return `${stem}_${modName}_tests.rs`;
}

function dedent(body, bodyStart, code, strip) {
  const pad = " ".repeat(strip);
  let offset = bodyStart;
  return body
    .split("\n")
    .map((line) => {
      const inCode = code[offset] === 1;
      offset += line.length + 1;
      if (!inCode) return line;
      if (line.startsWith(pad)) return line.slice(strip);
      return strip === 4 && line.startsWith("\t") ? line.slice(1) : line;
    })
    .join("\n");
}

// Crate roots and `mod.rs` own their directory; any other file `foo.rs` owns `foo/`.
const MOD_RS_STEMS = new Set(["mod", "lib", "main"]);

/**
 * Externalize every inline test module in `src`.
 *
 * `stem` is the source file's name without `.rs`; `taken` holds sibling file
 * names that already exist. A module nested in other inline modules (the
 * `mod imp { ... }` wrapper around platform code) gets a `#[path]` that climbs
 * out of the directories those modules imply. Returns the rewritten source,
 * the moved bodies (`{ name, fileName, body, line }`) and `skipped` modules
 * that need a human.
 */
export function externalizeSource(src, stem, taken = new Set()) {
  const code = codeMask(src);
  const skel = skeleton(src, code);
  const lines = src.split("\n");
  const lineStarts = [];
  let at = 0;
  for (const line of lines) {
    lineStarts.push(at);
    at += line.length + 1;
  }
  const lineOf = (index) => {
    let lo = 0;
    let hi = lineStarts.length - 1;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (lineStarts[mid] <= index) lo = mid;
      else hi = mid - 1;
    }
    return lo;
  };

  const moves = [];
  const skipped = [];
  const edits = [];
  const regions = [];
  const used = new Set(taken);

  const inlineMods = [...skel.matchAll(MOD_OPEN)].map((m) => {
    const open = m.index + m[0].length - 1;
    return { m, open, close: matchBrace(skel, open) };
  });
  const enclosing = (index) => inlineMods.filter((o) => o.open < index && o.close > index);

  // Collect the test modules first, then move the top-level ones before the nested.
  const candidates = [];
  for (const { m, open, close } of inlineMods) {
    const line = lineOf(m.index);
    // Walk up over the attributes and comments that belong to this `mod`,
    // joining a multi-line attribute into one logical entry for the cfg check.
    const attrs = [];
    const logical = [];
    let first = line;
    for (let k = line - 1; k >= 0; k -= 1) {
      const text = lines[k].trim();
      if (text.startsWith("//")) {
        attrs.unshift(lines[k]);
        first = k;
      } else if (text.startsWith("#[") || text.endsWith("]")) {
        let start = k;
        while (start > 0 && !lines[start].trim().startsWith("#[")) start -= 1;
        if (!lines[start].trim().startsWith("#[")) break;
        attrs.unshift(...lines.slice(start, k + 1));
        logical.unshift(lines.slice(start, k + 1).map((l) => l.trim()).join(" "));
        first = start;
        k = start;
      } else {
        break;
      }
    }
    if (logical.some((a) => CFG_TEST_ATTR.test(a) && !CFG_NOT_TEST.test(a))) {
      candidates.push({ m, open, close, line, attrs, first });
    }
  }
  candidates.sort((a, b) => (a.m[1] === "" ? 0 : 1) - (b.m[1] === "" ? 0 : 1) || a.m.index - b.m.index);

  for (const { m, open, close, line, attrs, first } of candidates) {
    const indent = m[1];
    if (regions.some(([from, to]) => m.index > from && m.index < to)) continue;
    const reject = (reason) => skipped.push({ line: line + 1, reason: `\`mod ${m[3]}\`: ${reason}` });
    const chain = enclosing(m.index);
    if (indent !== "" && (indent.length !== 4 * chain.length || chain.some((o) => o.close < 0))) {
      reject("nested somewhere other than directly inside inline modules; move it by hand");
      continue;
    }
    if (close < 0) {
      reject("unbalanced braces");
      continue;
    }
    const openEol = src.indexOf("\n", open);
    const closeLineStart = src.lastIndexOf("\n", close - 1) + 1;
    let closeEol = src.indexOf("\n", close);
    if (closeEol < 0) closeEol = src.length;
    if (src.slice(open + 1, openEol).trim() !== "" || src.slice(closeLineStart, close).trim() !== "") {
      reject("code shares a line with a brace");
      continue;
    }
    if (src.slice(close + 1, closeEol).trim() !== "") {
      reject("code follows the closing brace");
      continue;
    }
    const bodyStart = openEol + 1;
    if (closeLineStart <= bodyStart || src.slice(bodyStart, closeLineStart).trim() === "") {
      reject("empty module; delete it");
      continue;
    }
    const bodySkel = skel.slice(bodyStart, closeLineStart);
    if (/#\[path\s*=/.test(bodySkel) || /^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?mod[ \t]+\w+[ \t]*;/m.test(bodySkel)) {
      reject("declares out-of-line modules whose paths would shift");
      continue;
    }
    const fileName = testFileName(stem, m[3]);
    if (used.has(fileName)) {
      reject(`${fileName} already exists`);
      continue;
    }
    used.add(fileName);

    const climb = chain.length === 0 ? 0 : chain.length + (MOD_RS_STEMS.has(stem) ? 0 : 1);
    let body = dedent(src.slice(bodyStart, closeLineStart), bodyStart, code, indent.length + 4).replace(/^\n+/, "");
    body = `${body.replace(/\s+$/, "")}\n`;
    const declaration = [...attrs, `${indent}#[path = "${"../".repeat(climb)}${fileName}"]`, `${indent}${m[2]};`].join("\n");
    edits.push({
      start: lineStarts[first],
      end: closeEol < src.length ? closeEol + 1 : closeEol,
      text: `${declaration}\n`,
    });
    regions.push([m.index, close]);
    moves.push({ name: m[3], fileName, body, line: line + 1 });
  }

  let out = src;
  for (const e of edits.sort((a, b) => b.start - a.start)) {
    out = out.slice(0, e.start) + e.text + out.slice(e.end);
  }
  return { source: out, moves, skipped };
}

function editionFor(file, root) {
  const rootManifest = path.join(root, "Cargo.toml");
  for (let dir = path.dirname(file); ; dir = path.dirname(dir)) {
    const manifest = path.join(dir, "Cargo.toml");
    if (fs.existsSync(manifest)) {
      const toml = fs.readFileSync(manifest, "utf8");
      const own = toml.match(/^edition\s*=\s*"(\d{4})"/m);
      if (own) return own[1];
      if (/^edition\.workspace\s*=\s*true/m.test(toml) && fs.existsSync(rootManifest)) {
        const shared = fs.readFileSync(rootManifest, "utf8").match(/^edition\s*=\s*"(\d{4})"/m);
        if (shared) return shared[1];
      }
    }
    if (dir === root || dir === path.dirname(dir)) return "2021";
  }
}

function trackedRustFiles(root) {
  const out = execFileSync("git", ["-C", root, "ls-files", "-z", "--", "*.rs"], {
    encoding: "utf8",
    maxBuffer: 1 << 28,
  });
  return out
    .split("\0")
    .filter(Boolean)
    .filter((file) => !file.split("/").slice(0, -1).some((dir) => SKIPPED_DIRS.has(dir)));
}

function formatFiles(files, root) {
  const byEdition = new Map();
  for (const file of files) {
    const edition = editionFor(file, root);
    byEdition.set(edition, [...(byEdition.get(edition) ?? []), file]);
  }
  for (const [edition, group] of byEdition) {
    try {
      execFileSync("rustfmt", ["--edition", edition, ...group], { stdio: "pipe" });
    } catch (error) {
      console.warn(`  rustfmt (edition ${edition}) failed: ${String(error.stderr ?? error.message).split("\n")[0]}`);
    }
  }
}

function run(roots, { write, fmt }) {
  let movedTotal = 0;
  let manual = 0;
  for (const rootArg of roots) {
    const root = path.resolve(rootArg);
    const written = [];
    let moved = 0;
    for (const rel of trackedRustFiles(root)) {
      const file = path.join(root, rel);
      if (!fs.existsSync(file)) continue; // deleted in the working tree, not yet staged
      const src = fs.readFileSync(file, "utf8");
      if (!/#\[cfg\([^\n]*\btest\b/.test(src) && !/\]\s*\n\s*(?:pub[^\n]*\s)?mod\s/.test(src)) continue;
      const base = path.basename(file);
      if (base.endsWith("_tests.rs") || TEST_FILE_NAMES.has(base)) {
        if (/^[ \t]*#\[cfg\(test\)\]\s*\n\s*mod\s+\w+\s*\{/m.test(src)) {
          manual += 1;
          console.log(`manual ${rel}: inline module inside a test file`);
        }
        continue;
      }
      const dir = path.dirname(file);
      const taken = new Set(fs.readdirSync(dir));
      const result = externalizeSource(src, base.replace(/\.rs$/, ""), taken);
      for (const s of result.skipped) {
        manual += 1;
        console.log(`manual ${rel}:${s.line}: ${s.reason}`);
      }
      if (result.moves.length === 0) continue;
      moved += result.moves.length;
      if (write) {
        for (const move of result.moves) {
          fs.writeFileSync(path.join(dir, move.fileName), move.body);
          written.push(path.join(dir, move.fileName));
        }
        fs.writeFileSync(file, result.source);
      } else {
        for (const move of result.moves) console.log(`move   ${rel}:${move.line} mod ${move.name} -> ${move.fileName}`);
      }
    }
    if (write && fmt && written.length > 0) formatFiles(written, root);
    console.log(`${root}: ${moved} inline test module(s) ${write ? "moved" : "movable"}`);
    movedTotal += moved;
  }
  if (!write && movedTotal > 0) console.log("dry run; pass --write to apply");
  return write ? manual : manual + movedTotal;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const roots = args.filter((a) => !a.startsWith("--"));
  if (roots.length === 0) {
    console.error("usage: externalize-inline-tests.mjs <root>... [--write] [--no-fmt]");
    process.exit(2);
  }
  const remaining = run(roots, { write: args.includes("--write"), fmt: !args.includes("--no-fmt") });
  process.exit(remaining > 0 ? 1 : 0);
}
