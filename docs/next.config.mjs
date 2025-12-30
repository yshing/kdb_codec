import { createMDX } from 'fumadocs-mdx/next';

const withMDX = createMDX();

// Check if we're building for GitHub Pages
const isGitHubPages = process.env.GITHUB_ACTIONS === 'true';

/** @type {import('next').NextConfig} */
const config = {
  reactStrictMode: true,
  output: 'export',
  // GitHub Pages uses the repo name as basePath
  basePath: isGitHubPages ? '/kdb_codec' : '',
  images: {
    unoptimized: true,
  },
  trailingSlash: true,
};

export default withMDX(config);
