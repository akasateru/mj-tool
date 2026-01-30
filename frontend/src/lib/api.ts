import type {
  AnalyzeHandRequest,
  AnalyzeHandResponse,
  AnalyzeImageResponse,
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

/** 画像から手牌を読み取り、可能なら翻・符・役を取得。読み取った手牌は常に返す（手動修正用） */
export async function apiAnalyzeImage(
  image: File,
  options: { riichi?: boolean; tsumo?: boolean },
): Promise<
  | { ok: true; data: AnalyzeImageResponse }
  | { ok: false; status: number; message?: string }
> {
  const form = new FormData();
  form.append("image", image);
  form.append("riichi", options.riichi ? "true" : "false");
  form.append("tsumo", options.tsumo !== false ? "true" : "false");

  const res = await fetch(`${API_BASE}/analyze-image`, {
    method: "POST",
    body: form,
    cache: "no-store",
  });

  if (res.ok) {
    const data = (await res.json()) as AnalyzeImageResponse;
    return { ok: true, data };
  }

  let body: unknown;
  try {
    body = await res.json();
  } catch {
    body = {};
  }
  const detail =
    body && typeof body === "object" && "detail" in body && typeof (body as { detail: unknown }).detail === "string"
      ? (body as { detail: string }).detail
      : res.statusText || "画像解析に失敗しました";
  return { ok: false, status: res.status, message: detail };
}

