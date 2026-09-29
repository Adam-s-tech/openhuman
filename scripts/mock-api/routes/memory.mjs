import { json } from "../http.mjs";
import { behavior } from "../state.mjs";

/**
 * Mock of the TinyHumans backend's hosted CortexDB proxy (`/memory/*`), the
 * surface the core's `tinyhumans` memory engine speaks (tinymemory-remote,
 * `CortexWire::TinyHumans`).
 *
 * Every response is the backend's `{success,data}` envelope; failures are
 * `{success:false,error,errorCode}`. Each bearer token owns a separate
 * append-only event log, mirroring the real per-user isolation.
 *
 * Behaviour switches (via `setMockBehavior`):
 *   memoryForceStatus  "402" | "401" | "429" | ...  every /memory/* call fails
 *                      with that status and its canonical errorCode.
 *
 * Like the real memory API:
 *   - a reused `Idempotency-Key` header is a 409 `CONFLICT`, never forwarded;
 *   - a reused body `idempotency_key` with different text is a 409
 *     `IDEMPOTENCY_CONFLICT`, and forgetting does not release the key;
 *   - `GET /memory/scopes` honours `limit` and caps at 50 without it;
 *   - `/memory/events` lists newest first with every record emitted twice.
 */

const DEFAULT_SCOPE_PAGE = 50;

const ANSWER_KEYS = new Set([
  "scope",
  "question",
  "question_type",
  "question_date",
  "temporal",
  "filters",
  "answer_max_tokens",
  "answer_instructions",
  "cite_sources",
  "include_context",
  "use_pack_id",
]);

/** token -> { events, idempotency, claims, nextOffset, nextId } */
const stores = new Map();

export function resetMockMemory() {
  stores.clear();
}

function storeFor(token) {
  let store = stores.get(token);
  if (!store) {
    store = {
      events: [],
      idempotency: new Map(),
      claims: new Set(),
      nextOffset: 0,
      nextId: 0,
    };
    stores.set(token, store);
  }
  return store;
}

const STATUS_CODES = {
  401: "UNAUTHORIZED",
  402: "USER_INSUFFICIENT_CREDITS",
  403: "FORBIDDEN",
  409: "CONFLICT",
  429: "RATE_LIMITED",
  503: "UNAVAILABLE",
};

function fail(res, status, code, error) {
  json(res, status, {
    success: false,
    error: error || `failed: ${code}`,
    errorCode: code,
  });
}

function ok(res, data, status = 200) {
  json(res, status, { success: true, data });
}

function bearerOf(req) {
  const header = String(req.headers?.authorization || "");
  return header.startsWith("Bearer ") ? header.slice(7).trim() : "";
}

function queryOf(url) {
  const index = url.indexOf("?");
  return new URLSearchParams(index === -1 ? "" : url.slice(index + 1));
}

function textOf(event) {
  return event?.content?.text;
}

export async function handleMemory(ctx) {
  const { method, url, res, req, parsedBody } = ctx;
  const path = url.split("?")[0];
  if (!path.startsWith("/memory/")) return false;
  const route = path.slice("/memory/".length).replace(/\/+$/, "");
  const known = new Set([
    "experience",
    "events",
    "recall",
    "forget",
    "scopes",
    "answer",
  ]);
  if (!known.has(route)) return false;

  const token = bearerOf(req);
  if (!token) {
    fail(res, 401, "UNAUTHORIZED", "missing bearer");
    return true;
  }
  const forced = Number(behavior().memoryForceStatus || 0);
  if (forced >= 400) {
    fail(res, forced, STATUS_CODES[forced] || "UPSTREAM_ERROR", "forced by mock");
    return true;
  }

  const store = storeFor(token);
  const body = parsedBody && typeof parsedBody === "object" ? parsedBody : {};

  if (route === "experience" && method === "POST") {
    const key = String(body.idempotency_key || "").trim();
    if (!key) {
      fail(res, 400, "MISSING_IDEMPOTENCY_KEY", "idempotency_key is required");
      return true;
    }
    const claim = req.headers?.["idempotency-key"];
    if (claim) {
      if (store.claims.has(claim)) {
        fail(res, 409, "CONFLICT", "already claimed");
        return true;
      }
      store.claims.add(claim);
    }
    const text = String(textOf(body) ?? "");
    const seen = store.idempotency.get(key);
    if (seen) {
      const scope = String(body.scope || "");
      const modality = String(body.modality || "");
      const content = body.content ?? {};
      if (
        seen.text !== text ||
        seen.scope !== scope ||
        seen.modality !== modality ||
        JSON.stringify(seen.content) !== JSON.stringify(content)
      ) {
        fail(res, 409, "IDEMPOTENCY_CONFLICT", "idempotency key reused");
        return true;
      }
      ok(res, { event_id: seen.id, replayed_from_idempotency: true }, 200);
      return true;
    }
    store.nextOffset += 2;
    store.nextId += 1;
    const id = `evt_${store.nextId}`;
    store.idempotency.set(key, { text, id });
    store.events.push({
      id,
      scope,
      modality,
      wal_offset: store.nextOffset,
      content,
      context: { recorded_at: new Date().toISOString() },
    });
    ok(res, { event_id: id, status: "captured", replayed_from_idempotency: false });
    return true;
  }

  if (route === "events" && method === "GET") {
    const params = queryOf(url);
    const scope = params.get("scope") || "";
    const cursor = Number(params.get("cursor") || 0) || 0;
    const limit = Number(params.get("limit") || 50) || 50;
    const stream = [];
    for (const event of [...store.events].reverse()) {
      if (event.scope !== scope) continue;
      stream.push(event, event);
    }
    const items = stream.slice(cursor, cursor + limit);
    const next = cursor + items.length;
    ok(res, {
      items,
      has_more: next < stream.length,
      next_cursor: String(next),
    });
    return true;
  }

  if (route === "recall" && method === "POST") {
    const scope = String(body.scope || "");
    const query = String(body.query || "").toLowerCase();
    const hits = store.events
      .filter((e) => e.scope === scope)
      .filter((e) => !query || String(textOf(e) ?? "").toLowerCase().includes(query))
      .map((e) => {
        const text = textOf(e);
        return typeof text === "string"
          ? { ...e, content: { ...e.content, text: `[user] ${text}` } }
          : e;
      });
    ok(res, { pack_id: "pack_test", layers: { events: hits } });
    return true;
  }

  if (route === "forget" && method === "POST") {
    const scope = String(body.scope || "");
    const ids = Array.isArray(body.selector?.memory_ids)
      ? body.selector.memory_ids.map(String)
      : [];
    const unsupported = ["about_subject", "about_entity", "predicate"].some(
      (f) => body.selector && body.selector[f] !== undefined,
    );
    if (unsupported) {
      fail(res, 400, "UNSUPPORTED_SELECTOR", "the memory mock supports only memory_ids selectors");
      return true;
    }
    const selective = ids.length > 0;
    if (selective && body.confirm_all === true) {
      fail(res, 400, "AMBIGUOUS_SELECTOR_CONFIRM_ALL");
      return true;
    }
    if (!selective && body.confirm_all !== true) {
      fail(res, 422, "EMPTY_SELECTOR_WITHOUT_CONFIRMATION");
      return true;
    }
    const before = store.events.length;
    const requestedIds = new Set(ids);
    store.events = selective
      ? store.events.filter((e) => e.scope !== scope || !requestedIds.has(e.id))
      : store.events.filter((e) => e.scope !== scope);
    const deleted = before - store.events.length;
    ok(res, {
      deleted: { events: deleted },
      requested: ids.length,
      matched: deleted,
    });
    return true;
  }

  if (route === "scopes" && method === "GET") {
    const limit = Number(queryOf(url).get("limit") || DEFAULT_SCOPE_PAGE) || DEFAULT_SCOPE_PAGE;
    const paths = [...new Set(store.events.map((e) => e.scope))].sort();
    ok(res, { items: paths.slice(0, limit).map((path) => ({ path })) });
    return true;
  }

  if (route === "answer" && method === "POST") {
    const unknown = Object.keys(body).find((k) => !ANSWER_KEYS.has(k));
    if (unknown) {
      fail(res, 400, "VALIDATION_ERROR", `unknown key ${unknown}`);
      return true;
    }
    ok(res, {
      answer: "grounded",
      citations: [{ id: "evt_1", key: "k", content: "c", score: 0.5 }],
      context_block: "c",
      diagnostics: { answer_model: "mock" },
    });
    return true;
  }

  return false;
}
