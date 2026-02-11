/** @type {import('next').NextConfig} */
const nextConfig = {
    output: "standalone",
    async rewrites() {
        const apiUrl = process.env.API_URL || "http://gateway:8080";
        console.log("Checking API_URL:", process.env.API_URL, "Using:", apiUrl);
        return [
            {
                source: "/api/:path*",
                destination: `${apiUrl}/api/:path*`,
            },
            {
                source: "/metrics",
                destination: `${apiUrl}/metrics`,
            },
        ];
    },
};

export default nextConfig;
