/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docsSidebar: [
    'intro',
    'quickstart',
    {
      type: 'category',
      label: 'Platforms',
      items: ['platforms/windows', 'platforms/linux', 'platforms/macos'],
    },
    'troubleshooting',
  ],
};

export default sidebars;
