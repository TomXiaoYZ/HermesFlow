// Jupiter API 中继 - Vercel Edge Function (修复版)
export const config = {
    runtime: 'edge',
};

export default async function handler(req) {
    // CORS 预检
    if (req.method === 'OPTIONS') {
        return new Response(null, {
            status: 200,
            headers: {
                'Access-Control-Allow-Origin': '*',
                'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
                'Access-Control-Allow-Headers': 'Content-Type',
            },
        });
    }

    try {
        // 从请求中提取查询字符串
        const queryStart = req.url.indexOf('?');
        const queryString = queryStart !== -1 ? req.url.substring(queryStart) : '';

        // 构建 Jupiter URL
        const jupiterUrl = `https://quote-api.jup.ag/v6/quote${queryString}`;

        // 转发请求
        const res = await fetch(jupiterUrl);
        const text = await res.text();

        return new Response(text, {
            status: res.status,
            headers: {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*',
            },
        });
    } catch (err) {
        return new Response(
            JSON.stringify({
                error: String(err),
                message: err?.message || 'Unknown error'
            }),
            {
                status: 500,
                headers: {
                    'Content-Type': 'application/json',
                    'Access-Control-Allow-Origin': '*',
                },
            }
        );
    }
}
