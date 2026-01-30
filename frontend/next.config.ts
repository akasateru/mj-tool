import path from "node:path";
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // ルートに package.json があるとモジュール解決がルートに向くのを防ぐ
  turbopack: {
    root: path.resolve(__dirname),
  },
};

export default nextConfig;
