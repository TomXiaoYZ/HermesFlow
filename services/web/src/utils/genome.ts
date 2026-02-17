// Dynamic Maps - Populated by loadFactorConfig()
export let FEATURE_MAP: Record<number, string> = {
    // Fallback: equity-focused factor set
    0: "Return",
    1: "VwapDev",
    2: "VolRatio",
    3: "MeanRev",
    4: "AdvRatio",
    5: "Volatility",
    6: "Momentum",
    7: "RSI",
    8: "ClosePos",
    9: "Range",
};

let OP_OFFSET = 6; // Dynamically updated

export let OP_MAP: Record<number, { name: string; arity: number }> = {
    // Will be re-mapped based on OP_OFFSET
};

// Initialize operators based on offset
function initOperators(offset: number) {
    OP_OFFSET = offset;
    OP_MAP = {
        // Arity 2 (binary)
        [offset + 0]: { name: "+", arity: 2 },
        [offset + 1]: { name: "-", arity: 2 },
        [offset + 2]: { name: "*", arity: 2 },
        [offset + 3]: { name: "/", arity: 2 },
        // Arity 1 (unary)
        [offset + 4]: { name: "Neg", arity: 1 },
        [offset + 5]: { name: "Abs", arity: 1 },
        [offset + 6]: { name: "Sign", arity: 1 },
        // Arity 3 (ternary)
        [offset + 7]: { name: "Gate", arity: 3 },
        // Arity 1 (unary) - continued
        [offset + 8]: { name: "SPow", arity: 1 },
        [offset + 9]: { name: "DecayLin", arity: 1 },
        [offset + 10]: { name: "Delay1", arity: 1 },
        [offset + 11]: { name: "Delay5", arity: 1 },
        [offset + 12]: { name: "TsMean", arity: 1 },
        [offset + 13]: { name: "TsStd", arity: 1 },
        [offset + 14]: { name: "TsRank", arity: 1 },
        [offset + 15]: { name: "TsSum", arity: 1 },
        // Arity 2 (binary)
        [offset + 16]: { name: "TsCorr", arity: 2 },
        // Arity 1 (unary) - continued
        [offset + 17]: { name: "TsMin", arity: 1 },
        [offset + 18]: { name: "TsMax", arity: 1 },
        [offset + 19]: { name: "Log", arity: 1 },
        [offset + 20]: { name: "Sqrt", arity: 1 },
        [offset + 21]: { name: "TsArgmax", arity: 1 },
        [offset + 22]: { name: "TsDelta", arity: 1 },
    };
}

// Initial call with default offset (10 equity factors)
initOperators(10);

// Cache factor configs per exchange to avoid repeated fetches
const factorConfigCache: Record<string, boolean> = {};

export async function loadFactorConfig() {
    try {
        const res = await fetch("/api/v1/config/factors");
        if (res.ok) {
            applyFactorConfig(await res.json());
        }
    } catch {
        // Factor config unavailable, using defaults
    }
}

export async function loadFactorConfigForExchange(exchange: string) {
    if (factorConfigCache[exchange]) return;
    try {
        const res = await fetch(`/api/v1/evolution/${exchange}/config/factors`);
        if (res.ok) {
            applyFactorConfig(await res.json());
            factorConfigCache[exchange] = true;
        }
    } catch {
        // Factor config unavailable for exchange, using defaults
    }
}

function applyFactorConfig(config: { active_factors?: { id: number; name: string }[] }) {
    if (!config.active_factors) return;
    FEATURE_MAP = {};
    config.active_factors.forEach((f) => {
        FEATURE_MAP[f.id] = f.name;
    });
    initOperators(config.active_factors.length);
}


export function decodeGenome(tokens: number[]): string {
    if (!tokens || tokens.length === 0) return "Empty Strategy";

    const stack: string[] = [];

    for (const t of tokens) {
        if (t < OP_OFFSET) {  // Features are tokens 0..(OP_OFFSET-1)
            stack.push(FEATURE_MAP[t] || `F${t}`);
        } else {
            const op = OP_MAP[t] || { name: `Op${t}`, arity: 1 };

            if (op.arity === 2) {
                const b = stack.pop() || "?";
                const a = stack.pop() || "?";
                stack.push(`(${a} ${op.name} ${b})`);
            } else if (op.arity === 3) {
                const c = stack.pop() || "?";
                const b = stack.pop() || "?";
                const a = stack.pop() || "?";
                stack.push(`If(${a} > 0, ${b}, ${c})`);
            } else {
                const a = stack.pop() || "?";
                // Remove parens for common unary ops to reduce noise? No, keep it clear.
                stack.push(`${op.name}(${a})`);
            }
        }
    }

    return stack[0] || "Invalid Formula";
}

export function getFeatureImportance(tokens: number[]): Record<string, number> {
    const counts: Record<string, number> = {};
    for (const t of tokens) {
        if (t < OP_OFFSET) {  // Features are tokens 0..(OP_OFFSET-1)
            const name = FEATURE_MAP[t] || `F${t}`;
            counts[name] = (counts[name] || 0) + 1;
        }
    }
    return counts;
}
