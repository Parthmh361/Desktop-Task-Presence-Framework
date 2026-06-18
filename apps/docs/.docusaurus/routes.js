import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/',
    component: ComponentCreator('/', 'd3e'),
    routes: [
      {
        path: '/',
        component: ComponentCreator('/', '276'),
        routes: [
          {
            path: '/',
            component: ComponentCreator('/', 'eff'),
            routes: [
              {
                path: '/platforms/linux',
                component: ComponentCreator('/platforms/linux', 'dee'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/platforms/macos',
                component: ComponentCreator('/platforms/macos', 'ae0'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/platforms/windows',
                component: ComponentCreator('/platforms/windows', 'eb7'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/quickstart',
                component: ComponentCreator('/quickstart', 'cee'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/troubleshooting',
                component: ComponentCreator('/troubleshooting', 'c3d'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/',
                component: ComponentCreator('/', 'b56'),
                exact: true,
                sidebar: "docsSidebar"
              }
            ]
          }
        ]
      }
    ]
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];
