import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'The Headstash Contracts',
  tagline: 'A transparency minimizing, multi-chain airdrop protocol.',
  favicon: 'img/logo.svg',

  // Set the production url of your site here
  url: 'https://permissionlessweb.github.io',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'permissionlessweb', // Usually your GitHub org/user name.
  projectName: 'cw-infuser', // Usually your repo name.

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/permissionlessweb/cw-infuser/tree/feat/docusaurus-docs/docs',
          // Routed the docs to the root path
          routeBasePath: "/",
          sidebarCollapsed: false,
          // Select the latest version
          lastVersion: "current",
          versions: {
            current: {
              label: 'main',
              path: 'main',
              banner: 'unreleased',
            },
            // "v0.4.x": {
            //   label: 'v0.4',
            //   path: 'v0.4',
            // },
          },
        },
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themes: [
    'docusaurus-theme-github-codeblock',
    [
      require.resolve("@easyops-cn/docusaurus-search-local"),
      /** @type {import("@easyops-cn/docusaurus-search-local").PluginOptions} */
      ({
        // ... Your options.
        // `hashed` is recommended as long-term-cache of index file is possible.
        hashed: true,
        // For Docs using Chinese, The `language` is recommended to set to:
        // ```
        // language: ["en", "zh"],
        // ```
      }),
    ],
  ],

  themeConfig: {
    // Replace with your project's social card
    image: 'https://opengraph.githubassets.com/946cd03a2431502cb8cdbb579ca48c56e2c4060ca5ff0b25e13739f3fc08b512/permissionlessweb/cw-headstash',
    navbar: {
      title: 'Cw-Infuser',
      logo: {
        alt: 'Logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          type: "docsVersionDropdown",
          position: "right",
          dropdownActiveClassDisabled: true,
        },
        {
          href: 'https://github.com/permissionlessweb/cw-infuser',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Documentation',
              to: '/main',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'Discord',
              href: 'https://discordapp.com/invite/docusaurus',
            },
            {
              label: 'Twitter',
              href: 'https://twitter.com/terpculture',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/permissionlessweb/cw-infuser',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()}. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
    colorMode: {
      defaultMode: 'dark',
    },
    // github codeblock theme configuration
    codeblock: {
        showGithubLink: true,
        githubLinkLabel: 'View on GitHub',
        showRunmeLink: false,
        runmeLinkLabel: 'Checkout via Runme'
    },
  } satisfies Preset.ThemeConfig,
  plugins: [
    [
      "@docusaurus/plugin-client-redirects",
      {
        // makes the default page next in production
        redirects: [
          {
            from: ["/", "/master", "/next", "/docs"],
            to: "/main",
          },
        ],
      },
    ],
    async function myPlugin(context, options) {
      return {
        name: "docusaurus-tailwindcss",
        configurePostCss(postcssOptions) {
          postcssOptions.plugins.push(require("postcss-import"));
          postcssOptions.plugins.push(require("tailwindcss/nesting"));
          postcssOptions.plugins.push(require("tailwindcss"));
          postcssOptions.plugins.push(require("autoprefixer"));
          return postcssOptions;
        },
      };
    },
  ],
};

export default config;
