import type {NextConfig} from "next";

const nextConfig: NextConfig = {
    images: {
        unoptimized: true,
        remotePatterns: [new URL('http://localhost:9000/lot-images/**')]
    },
    output: 'standalone',
};

export default nextConfig;
