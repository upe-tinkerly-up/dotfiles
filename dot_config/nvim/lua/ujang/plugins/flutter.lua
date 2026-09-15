return {
  {
    "akinsho/flutter-tools.nvim",
    lazy = false,
    dependencies = {
      "nvim-lua/plenary.nvim",
      "stevearc/dressing.nvim",
    },
    config = function()
      require("flutter-tools").setup({
        flutter_path = vim.fn.expand("~/flutter/bin/flutter"),
        devtools = {
          autostart = false,
        },
        lsp = {
          on_attach = function(client, bufnr)
            -- keymaps LSP standar kamu taruh di sini
            local opts = { noremap = true, silent = true, buffer = bufnr }
            vim.keymap.set("n", "gd", vim.lsp.buf.definition, opts)
            vim.keymap.set("n", "K", vim.lsp.buf.hover, opts)
            vim.keymap.set("n", "<leader>rn", vim.lsp.buf.rename, opts)
            vim.keymap.set("n", "<leader>ca", vim.lsp.buf.code_action, opts)
            vim.keymap.set("n", "gr", vim.lsp.buf.references, opts)
          end,
          settings = {
            dart = {
              completeFunctionCalls = true,
              showTodos = true,
              renameFilesWithClasses = "prompt",
              enableSnippets = true,
              analysisExcludedFolders = {
                vim.fn.expand("~/flutter/packages"),
                vim.fn.expand("~/flutter/.pub-cache"),
              },
            },
          },
        },
      })
    end,
  },
}
