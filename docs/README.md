# kdb_codec Documentation

This is the documentation site for kdb_codec, built with [Fumadocs](https://fumadocs.dev) and [Next.js](https://nextjs.org).

## Development

To run the development server:

```bash
cd docs
npm install
npm run dev
```

Open http://localhost:3000 with your browser to see the result.

## Building

To build the static site:

```bash
npm run build
```

The static files will be generated in the `out` directory.

## Deployment

The documentation is automatically deployed to GitHub Pages when changes are pushed to the main branch. See `.github/workflows/docs.yml` for the deployment configuration.

## Documentation Structure

- `content/docs/` - MDX documentation files
  - `index.mdx` - Getting started guide
  - `codec-pattern.mdx` - Tokio codec pattern documentation
  - `qstream.mdx` - QStream client documentation
  - `k-macro.mdx` - K macro usage guide
  - `index-trait.mdx` - Index trait documentation
- `app/` - Next.js app router pages
- `lib/` - Shared utilities

## Learn More

- [Fumadocs Documentation](https://fumadocs.dev) - Learn about Fumadocs
- [Next.js Documentation](https://nextjs.org/docs) - Learn about Next.js
- [kdb_codec on crates.io](https://crates.io/crates/kdb_codec) - The Rust crate
