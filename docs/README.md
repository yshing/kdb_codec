# kdb_codec Documentation

This is the documentation site for kdb_codec, built with [VitePress](https://vitepress.dev/).

## Development

To run the development server with Bun:

```bash
cd docs
bun install
bun run dev
```

Or with npm:

```bash
cd docs
npm install
npm run dev
```

Open `http://localhost:5173` with your browser to see the result.

## Building

To build the static site:

```bash
bun run build
```

The static files will be generated in the `.vitepress/dist` directory.

## Deployment

The documentation is automatically deployed to GitHub Pages when changes are pushed to the main branch. See `.github/workflows/docs.yml` for the deployment configuration.

## Documentation Structure

- `guide/` - Documentation pages
  - `index.md` - Getting started guide
  - `installation.md` - Installation instructions
  - `codec-pattern.md` - Tokio codec pattern documentation
  - `qstream.md` - QStream client documentation
  - `k-macro.md` - K macro usage guide
  - `index-trait.md` - Index trait documentation
- `index.md` - Homepage
- `.vitepress/config.mts` - VitePress configuration

## Learn More

- [VitePress Documentation](https://vitepress.dev/) - Learn about VitePress
- [kdb_codec on crates.io](https://crates.io/crates/kdb_codec) - The Rust crate
