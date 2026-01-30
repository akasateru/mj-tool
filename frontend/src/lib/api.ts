import type {
  AnalyzeHandRequest,
  AnalyzeHandResponse,
  CalcResponse,
  Fact,
  HandRecord,
  HandSummary,
  RecalcResponse,
  ValidationError,
} from "./types";

const API_BASE =
  process.env.NEXT_PUBLIC_API_BASE_URL?.replace(/\/+$/, "") ??
  "http://localhost:8080";

async function fetchJson<T>(
  path: string,
  init?: RequestInit,
): Promise<{ ok: true; data: T } | { ok: false; status: number; errors?: ValidationError[]; message?: string }> {
  const res = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: {
      "content-type": "application/json",
      ...(init?.headers ?? {}),
    },
    cache: "no-store",
  });

  if (res.ok) {
    return { ok: true, data: (await res.json()) as T };
  }

  let body: unknown = undefined;
  try {
    body = await res.json();
  } catch {
    // ignore
  }
  if (
    res.status === 422 &&
    body &&
    typeof body === "object" &&
    "errors" in body &&
    Array.isArray((body as { errors: unknown }).errors)
  ) {
    return {
      ok: false,
      status: res.status,
      errors: (body as { errors: ValidationError[] }).errors,
    };
  }
  return { ok: false, status: res.status, message: JSON.stringify(body ?? {}) };
}

export async function apiCalc(fact: Fact) {
  return fetchJson<CalcResponse>("/calc", {
    method: "POST",
    body: JSON.stringify({ fact }),
  });
}

export async function apiCreateHand(input: {
  fact: Fact;
  memo?: string;
  tags?: string[];
}) {
  return fetchJson<HandRecord>("/hands", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export async function apiListHands(params?: { q?: string; limit?: number }) {
  const qs = new URLSearchParams();
  if (params?.q) qs.set("q", params.q);
  if (params?.limit) qs.set("limit", String(params.limit));
  const suffix = qs.toString() ? `?${qs.toString()}` : "";
  return fetchJson<HandSummary[]>(`/hands${suffix}`, { method: "GET" });
}

export async function apiGetHand(id: string) {
  return fetchJson<HandRecord>(`/hands/${encodeURIComponent(id)}`, {
    method: "GET",
  });
}

export async function apiRecalcHand(id: string) {
  return fetchJson<RecalcResponse>(`/hands/${encodeURIComponent(id)}/recalc`, {
    method: "POST",
  });
}

export async function apiAnalyzeHand(req: AnalyzeHandRequest) {
  return fetchJson<AnalyzeHandResponse>("/analyze-hand", {
    method: "POST",
    body: JSON.stringify(req),
  });
}

