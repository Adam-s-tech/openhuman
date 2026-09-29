import assert from "node:assert/strict";
import test from "node:test";

import { resetMockBehavior, setMockBehavior } from "../../state.mjs";
import { handleMemory, resetMockMemory } from "../memory.mjs";

function createRes() {
  return {
    statusCode: 0,
    headers: {},
    body: "",
    writeHead(status, headers = {}) {
      this.statusCode = status;
      this.headers = headers;
    },
    setHeader(name, value) {
      this.headers[name] = value;
    },
    end(chunk = "") {
      this.body += String(chunk);
    },
  };
}

async function call(method, url, body, headers = { authorization: "Bearer tok-a" }) {
  const res = createRes();
  const handled = await handleMemory({
    method,
    url,
    res,
    req: { headers },
    parsedBody: body ?? null,
  });
  return { handled, status: res.statusCode, json: res.body ? JSON.parse(res.body) : null };
}

const write = (scope, key, text, headers) =>
  call(
    "POST",
    "/memory/experience",
    { scope, idempotency_key: key, modality: "text", content: { text } },
    headers,
  );

test.beforeEach(() => {
  resetMockBehavior();
  resetMockMemory();
});

test("non-memory paths are not handled", async () => {
  const r = await call("GET", "/health");
  assert.equal(r.handled, false);
});

test("a missing bearer is a 401 UNAUTHORIZED envelope", async () => {
  const r = await call("GET", "/memory/scopes", null, {});
  assert.equal(r.status, 401);
  assert.equal(r.json.success, false);
  assert.equal(r.json.errorCode, "UNAUTHORIZED");
});

test("experience then recall round-trips, prefixing the speaker", async () => {
  const w = await write("ns/a", "k1", "the sky is blue");
  assert.equal(w.status, 200);
  assert.equal(w.json.data.event_id, "evt_1");
  const r = await call("POST", "/memory/recall", { scope: "ns/a", query: "SKY" });
  assert.equal(r.json.data.layers.events[0].content.text, "[user] the sky is blue");
});

test("stores are isolated per bearer token", async () => {
  await write("ns/a", "k1", "secret");
  const r = await call("GET", "/memory/scopes", null, { authorization: "Bearer tok-b" });
  assert.deepEqual(r.json.data.items, []);
});

test("a reused Idempotency-Key header is a 409 CONFLICT", async () => {
  const headers = { authorization: "Bearer tok-a", "idempotency-key": "claim-1" };
  assert.equal((await write("s", "k1", "one", headers)).status, 200);
  const again = await write("s", "k2", "two", headers);
  assert.equal(again.status, 409);
  assert.equal(again.json.errorCode, "CONFLICT");
});

test("a reused body key with different text is IDEMPOTENCY_CONFLICT", async () => {
  await write("s", "k1", "one");
  const r = await write("s", "k1", "two");
  assert.equal(r.status, 409);
  assert.equal(r.json.errorCode, "IDEMPOTENCY_CONFLICT");
});

test("scopes honours limit and caps at 50 without it", async () => {
  for (let i = 0; i < 60; i += 1) await write(`scope/${String(i).padStart(2, "0")}`, `k${i}`, "x");
  const all = await call("GET", "/memory/scopes?limit=100");
  assert.equal(all.json.data.items.length, 60);
  const capped = await call("GET", "/memory/scopes");
  assert.equal(capped.json.data.items.length, 50);
  const two = await call("GET", "/memory/scopes?limit=2");
  assert.equal(two.json.data.items.length, 2);
});

test("memoryForceStatus=402 fails every call with USER_INSUFFICIENT_CREDITS", async () => {
  setMockBehavior("memoryForceStatus", "402");
  const r = await write("s", "k1", "x");
  assert.equal(r.status, 402);
  assert.equal(r.json.errorCode, "USER_INSUFFICIENT_CREDITS");
});

test("forget refuses an empty selector without confirm_all", async () => {
  const r = await call("POST", "/memory/forget", { scope: "s", selector: {} });
  assert.equal(r.status, 422);
  await write("s", "k1", "x");
  const del = await call("POST", "/memory/forget", { scope: "s", selector: {}, confirm_all: true });
  assert.equal(del.json.data.deleted.events, 1);
});
