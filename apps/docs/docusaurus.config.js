/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'DTPF',
  tagline: 'Desktop Task Presence Framework',
  url: 'https://dtpf.dev',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  organizationName: 'dtpf',
  projectName: 'desktop-task-presence-framework',

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.js',
          routeBasePath: '/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      },
    ],
  ],

  themeConfig: {
    navbar: {
      title: 'DTPF',
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Docs',
        },
        {
          href: 'https://github.com/dtpf/desktop-task-presence-framework',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      copyright: `Copyright © ${new Date().getFullYear()} DTPF contributors. MIT License.`,
    },
  },
};

export default config;
