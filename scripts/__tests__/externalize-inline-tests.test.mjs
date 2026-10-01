import assert from "node:assert/strict";
import { test } from "node:test";

import { codeMask, externalizeSource } from "../externalize-inline-tests.mjs";

const DECL = '#[cfg(test)]\n#[path = "lib_tests.rs"]\nmod tests;\n';

test("moves a trailing inline module and dedents its body", () => {
  const src = "pub fn a() {}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn t() {\n        a();\n    }\n}\n";
  const out = externalizeSource(src, "lib");
  assert.equal(out.source, `pub fn a() {}\n\n${DECL}`);
  assert.equal(out.moves.length, 1);
  assert.equal(out.moves[0].fileName, "lib_tests.rs");
  assert.equal(out.moves[0].body, "use super::*;\n\n#[test]\nfn t() {\n    a();\n}\n");
  assert.deepEqual(out.skipped, []);
});

test("braces in strings, raw strings, chars and comments do not end the block", () => {
  const src = [
    "#[cfg(test)]",
    "mod tests {",
    '    const A: &str = "}";',
    '    const B: &str = r#"} mod x { "#;',
    "    const C: char = '}';",
    "    // }",
    "    /* } /* } */ } */",
    "    fn f<'a>(x: &'a str) -> &'a str { x }",
    "}",
    "fn after() {}",
    "",
  ].join("\n");
  const out = externalizeSource(src, "lib");
  assert.equal(out.moves[0].body.trimEnd().split("\n").length, 6);
  assert.equal(out.source, `${DECL}fn after() {}\n`);
});

test("does not dedent lines that sit inside a multi-line string", () => {
  const src = '#[cfg(test)]\nmod tests {\n    const S: &str = "a\n    b";\n}\n';
  assert.equal(externalizeSource(src, "lib").moves[0].body, 'const S: &str = "a\n    b";\n');
});

test("ignores a `mod tests {` that only appears inside a string or comment", () => {
  const src = 'const S: &str = r#"\n#[cfg(test)]\nmod tests {\n}\n"#;\n// #[cfg(test)]\n// mod tests {\n';
  const out = externalizeSource(src, "lib");
  assert.equal(out.moves.length, 0);
  assert.equal(out.source, src);
});

test("keeps sibling attributes and names extra modules after the module", () => {
  const src = "#[cfg(test)]\n#[allow(clippy::unwrap_used)]\nmod tests {\n    fn a() {}\n}\n\n#[cfg(test)]\nmod fixtures {\n    fn b() {}\n}\n";
  const out = externalizeSource(src, "lib");
  assert.deepEqual(out.moves.map((m) => m.fileName), ["lib_tests.rs", "lib_fixtures_tests.rs"]);
  assert.match(out.source, /#\[allow\(clippy::unwrap_used\)\]\n#\[path = "lib_tests.rs"\]\nmod tests;/);
  assert.match(out.source, /#\[path = "lib_fixtures_tests.rs"\]\nmod fixtures;/);
});

test("reads a multi-line attribute, and leaves a non-test module behind one alone", () => {
  const lint = "#[allow(\n    missing_docs,\n    reason = \"generated\"\n)]\npub(crate) mod exports {\n    fn a() {}\n}\n";
  const plain = externalizeSource(lint, "lib");
  assert.equal(plain.moves.length, 0);
  assert.deepEqual(plain.skipped, []);

  const gated = "#[cfg(test)]\n#[allow(\n    missing_docs,\n    reason = \"fixture\"\n)]\nmod tests {\n    fn a() {}\n}\n";
  const out = externalizeSource(gated, "lib");
  assert.equal(out.moves.length, 1);
  assert.equal(
    out.source,
    '#[cfg(test)]\n#[allow(\n    missing_docs,\n    reason = "fixture"\n)]\n#[path = "lib_tests.rs"]\nmod tests;\n',
  );
});

test("leaves cfg(not(test)) and plain inline modules alone", () => {
  const src = "#[cfg(not(test))]\nmod real {\n    fn a() {}\n}\nmod plain {\n    fn b() {}\n}\n";
  const out = externalizeSource(src, "lib");
  assert.equal(out.moves.length, 0);
  assert.equal(out.source, src);
});

test("writes a module nested in an inline wrapper to the directory rustc implies", () => {
  const src = "#[cfg(target_os = \"macos\")]\nmod imp {\n    fn a() {}\n\n    #[cfg(test)]\n    mod tests {\n        use super::*;\n\n        #[test]\n        fn t() {}\n    }\n}\n";
  const plain = externalizeSource(src, "cf_type");
  assert.equal(plain.moves[0].fileName, "cf_type_tests.rs");
  assert.equal(plain.moves[0].relDir, "cf_type/imp");
  assert.equal(plain.moves[0].body, "use super::*;\n\n#[test]\nfn t() {}\n");
  assert.match(plain.source, /    #\[cfg\(test\)\]\n    #\[path = "cf_type_tests.rs"\]\n    mod tests;\n}\n$/);
  assert.deepEqual(plain.skipped, []);
  // A `mod.rs` or crate root owns its directory, so the wrapper's directory sits beside it.
  assert.equal(externalizeSource(src, "mod").moves[0].relDir, "imp");
});

test("names a second nested module after its wrapper when the file name is taken", () => {
  const block = (wrapper) =>
    `mod ${wrapper} {\n    #[cfg(test)]\n    mod tests {\n        fn a() {}\n    }\n}\n`;
  const out = externalizeSource(`${block("imp")}\n${block("raise")}`, "ops");
  assert.deepEqual(out.moves.map((m) => m.fileName), ["ops_tests.rs", "ops_raise_tests.rs"]);
  assert.deepEqual(out.moves.map((m) => m.relDir), ["ops/imp", "ops/raise"]);
  assert.deepEqual(out.skipped, []);
});

test("puts a bin crate root's tests in a subdirectory Cargo will not build as a binary", () => {
  const src = "fn main() {}\n\n#[cfg(test)]\nmod tests {\n    fn a() {}\n}\n";
  const out = externalizeSource(src, "tool", new Set(), { subdir: "tool" });
  assert.match(out.source, /#\[path = "tool\/tool_tests.rs"\]\nmod tests;/);
  assert.equal(out.moves[0].relDir, "tool");
});

test("reports a test module nested inside a function instead of moving it", () => {
  const src = "fn f() {\n    #[cfg(test)]\n    mod tests {\n        fn a() {}\n    }\n}\n";
  const out = externalizeSource(src, "lib");
  assert.equal(out.moves.length, 0);
  assert.equal(out.skipped.length, 1);
  assert.match(out.skipped[0].reason, /nested somewhere other than/);
});

test("refuses a body that declares out-of-line modules", () => {
  const src = '#[cfg(test)]\nmod tests {\n    #[path = "x.rs"]\n    mod x;\n}\n';
  const out = externalizeSource(src, "lib");
  assert.equal(out.moves.length, 0);
  assert.match(out.skipped[0].reason, /out-of-line/);
});

test("refuses to overwrite an existing test file", () => {
  const src = "#[cfg(test)]\nmod tests {\n    fn a() {}\n}\n";
  const out = externalizeSource(src, "lib", new Set(["lib_tests.rs"]));
  assert.equal(out.moves.length, 0);
  assert.match(out.skipped[0].reason, /already exists/);
});

test("codeMask treats lifetimes as code and char literals as data", () => {
  const src = "fn f<'a>(c: char) { let x = '{'; }";
  const mask = codeMask(src);
  assert.equal(mask[src.indexOf("'a")], 1);
  assert.equal(mask[src.indexOf("'{'") + 1], 0);
});
