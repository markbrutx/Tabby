import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Tabby',
  description: 'macOS-first terminal workspace app',
  base: '/Tabby/',

  head: [
    ['link', { rel: 'icon', type: 'image/png', href: '/Tabby/tabby-brand.png' }],
  ],

  themeConfig: {
    logo: '/tabby-brand.png',

    nav: [
      { text: 'Guide', link: '/guide/' },
      { text: 'Architecture', link: '/architecture/' },
      { text: 'Contributing', link: '/contributing/' },
      { text: 'About', link: '/about' },
      { text: 'GitHub', link: 'https://github.com/markbrutx/Tabby' },
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Guide',
          items: [
            { text: 'Getting Started', link: '/guide/' },
            { text: 'Installation', link: '/guide/installation' },
            { text: 'Features', link: '/guide/features' },
            { text: 'CLI Usage', link: '/guide/cli' },
            { text: 'FAQ & Troubleshooting', link: '/guide/faq' },
          ],
        },
      ],
      '/architecture/': [
        {
          text: 'Architecture',
          items: [
            { text: 'Overview', link: '/architecture/' },
            { text: 'Bounded Contexts', link: '/architecture/bounded-contexts' },
            { text: 'IPC Reference', link: '/architecture/ipc-reference' },
          ],
        },
        {
          text: 'ADRs',
          items: [
            { text: 'Index', link: '/architecture/adr/' },
            { text: 'ADR-001: Terminal Output Hot Path', link: '/architecture/adr/001-terminal-output-hot-path' },
          ],
        },
      ],
      '/contributing/': [
        {
          text: 'Contributing',
          items: [
            { text: 'Overview', link: '/contributing/' },
            { text: 'Development Setup', link: '/contributing/development' },
            { text: 'Coding Standards', link: '/contributing/coding-standards' },
          ],
        },
      ],
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/markbrutx/Tabby' },
    ],

    search: { provider: 'local' },

    footer: {
      message: 'Released under the MIT License.',
      copyright: `Copyright ${new Date().getFullYear()} Tabby Contributors`,
    },
  },
})
