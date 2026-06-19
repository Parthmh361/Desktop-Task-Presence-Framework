/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'DTPF',
  tagline: 'Desktop Task Presence Framework',
  url: 'https://parthmh361.github.io',
  baseUrl: '/Desktop-Task-Presence-Framework/',
  onBrokenLinks: 'throw',
  organizationName: 'Parthmh361',
  projectName: 'Desktop-Task-Presence-Framework',
  trailingSlash: false,

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
          href: 'https://github.com/Parthmh361/Desktop-Task-Presence-Framework',
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
