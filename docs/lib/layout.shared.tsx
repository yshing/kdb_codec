import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: 'kdb_codec',
    },
    githubUrl: 'https://github.com/yshing/kdb_codec',
    links: [
      {
        text: 'Docs',
        url: '/docs',
        active: 'nested-url',
      },
      {
        text: 'crates.io',
        url: 'https://crates.io/crates/kdb_codec',
      },
      {
        text: 'docs.rs',
        url: 'https://docs.rs/kdb_codec',
      },
    ],
  };
}
