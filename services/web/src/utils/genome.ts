
export const FEATURE_MAP: Record<number, string> = {
    0: "Return",
    1: "Liquidity",
    2: "Pressure",
    3: "FOMO",
    4: "Deviation",
    5: "LogVol",
    6: "VolCluster",
    7: "MomRev",
    8: "RSI",
    9: "ln(Open)",
    10: "ln(High)",
    11: "ln(Low)",
    12: "ln(Close)",
    13: "ln(Vol)",
};

export const OP_MAP: Record<number, { name: string; arity: number }> = {
    // Arity 2
    14: { name: "+", arity: 2 },
    15: { name: "-", arity: 2 },
    16: { name: "*", arity: 2 },
    17: { name: "/", arity: 2 },
    30: { name: "Corr", arity: 2 },
    // Arity 1
    18: { name: "Neg", arity: 1 },
    19: { name: "Abs", arity: 1 },
    20: { name: "Sign", arity: 1 },
    22: { name: "Jump", arity: 1 },
    23: { name: "Decay", arity: 1 },
    24: { name: "Delay", arity: 1 },
    25: { name: "Max3", arity: 1 },
    26: { name: "TsMean", arity: 1 },
    27: { name: "TsStd", arity: 1 },
    28: { name: "TsRank", arity: 1 },
    29: { name: "TsSum", arity: 1 },
    31: { name: "CsRank", arity: 1 },
    32: { name: "CsMean", arity: 1 },
    // Arity 3
    21: { name: "Gate", arity: 3 },
};

export function decodeGenome(tokens: number[]): string {
    if (!tokens || tokens.length === 0) return "Empty Strategy";

    const stack: string[] = [];

    for (const t of tokens) {
        if (t < 14) {
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
        if (t < 14) {
            const name = FEATURE_MAP[t] || `F${t}`;
            counts[name] = (counts[name] || 0) + 1;
        }
    }
    return counts;
}
