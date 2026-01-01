import { defineConfig } from 'vitepress'

// Check if we're building for GitHub Pages
const base = process.env.GITHUB_ACTIONS ? '/kdb_codec/' : '/'

export default defineConfig({
  title: 'kdb_codec',
  description: 'A Rust library for kdb+ IPC with true cancellation safety',
  base,
  
  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: `${base}logo.svg` }],
  ],

  themeConfig: {
    logo: '/logo.svg',
    
    nav: [
      { text: 'Guide', link: '/guide/' },
      { text: 'crates.io', link: 'https://crates.io/crates/kdb_codec' },
      { text: 'docs.rs', link: 'https://docs.rs/kdb_codec' },
    ],

    sidebar: [
      {
        text: 'Getting Started',
        items: [
          { text: 'Introduction', link: '/guide/' },
          { text: 'Installation', link: '/guide/installation' },
        ]
      },
      {
        text: 'Core Concepts',
        items: [
          { text: 'Codec Pattern', link: '/guide/codec-pattern' },
          { text: 'QStream Client', link: '/guide/qstream' },
        ]
      },
      {
        text: 'Data Construction',
        items: [
          { text: 'K Macro', link: '/guide/k-macro' },
          { text: 'Index Trait', link: '/guide/index-trait' },
        ]
      },
      {
        text: 'Reference',
        items: [
          { text: 'Type Mapping & Coverage', link: '/guide/type-mapping' },
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/yshing/kdb_codec' }
    ],

    search: {
      provider: 'local'
    },

    footer: {
      message: 'Released under the Apache-2.0 License.',
      copyright: 'Copyright © 2024 kdb_codec contributors'
    },

    editLink: {
      pattern: 'https://github.com/yshing/kdb_codec/edit/main/docs/:path',
      text: 'Edit this page on GitHub'
    }
  }
})
