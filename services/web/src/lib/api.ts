/**
 * Auth-aware API utilities.
 * Wraps fetch with JWT Bearer token from localStorage and handles 401 redirects.
 */

export function getToken(): string | null {
    if (typeof window === "undefined") return null;
    return localStorage.getItem("token");
}

/** Build WebSocket URL with JWT token as query parameter. */
export function getWsUrl(): string {
    const host =
        typeof window !== "undefined" ? window.location.hostname : "localhost";
    const base = `ws://${host}:8080/ws`;
    const token = getToken();
    return token ? `${base}?token=${encodeURIComponent(token)}` : base;
}

/** Fetch wrapper that injects Authorization header and redirects on 401. */
export async function authFetch(
    url: string,
    options?: RequestInit
): Promise<Response> {
    const token = getToken();
    const headers = new Headers(options?.headers);

    if (token && !headers.has("Authorization")) {
        headers.set("Authorization", `Bearer ${token}`);
    }

    const res = await fetch(url, { ...options, headers });

    if (res.status === 401 && typeof window !== "undefined") {
        localStorage.removeItem("token");
        localStorage.removeItem("lastActivity");
        window.location.href = "/login";
    }

    return res;
}
