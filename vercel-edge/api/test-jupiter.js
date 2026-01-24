// 测试能否从 Edge Function 访问 Jupiter
export const config = {
    runtime: 'edge',
};

export default async function handler(req) {
    try {
        const testUrl = 'https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=100000000';

        const res = await fetch(testUrl);
        const status = res.status;
        const text = await res.text();

        return new Response(`Status: ${status}, Body: ${text.substring(0, 200)}`, {
            headers: { 'Content-Type': 'text/plain' }
        });
    } catch (err) {
        return new Response(`Error: ${err.message}`, {
            headers: { 'Content-Type': 'text/plain' }
        });
    }
}
