// 最简测试版本 - 只返回固定内容
export const config = {
    runtime: 'edge',
};

export default async function handler(request) {
    return new Response('Edge Function is working!', {
        headers: { 'Content-Type': 'text/plain' }
    });
}
