return {
  {
    "williamboman/mason-lspconfig.nvim",
    opts = {
      -- list of servers for mason to install
      ensure_installed = {
        "ts_ls",
        "html",
        "cssls",
        "tailwindcss",
        "svelte",
        "lua_ls",
        "graphql",
        "emmet_ls",
        "prismals",
        "pyright",
        -- "eslint",
        -- "rust_analyzer",
        "gopls",
        "marksman",
        "markdownlint-cli2",
      },
    },
    dependencies = {
      {
        "williamboman/mason.nvim",
        opts = {
          ui = {
            icons = {
              package_installed = "✓",
              package_pending = "➜",
              package_uninstalled = "✗",
            },
          },
        },
      },
      "neovim/nvim-lspconfig",
    },
  },
  {
    "WhoIsSethDaniel/mason-tool-installer.nvim",
    opts = {
      ensure_installed = {
        "prettier", -- prettier formatter
        "stylua", -- lua formatter
        "isort", -- python formatter
        "black", -- python formatter
        "pylint",
        -- "eslint_d",
        -- "rustfmt",
        "gofumpt", -- go formatter (lebih strict dari gofmt)
        "goimports", -- otomatis manage imports
        "golangci-lint", -- go linter
        "delve", -- go debugger (untuk DAP / neotest)
      },
    },
    dependencies = {
      "williamboman/mason.nvim",
    },
  },
}
