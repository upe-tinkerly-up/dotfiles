return {
  "mrcjkb/rustaceanvim",
  version = "^8",
  ft = { "rust" },

  config = function()
    vim.diagnostic.config({
      virtual_text = {
        prefix = "●",
        source = "if_many",
      },
      signs = true,
      underline = true,
      update_in_insert = false,
      severity_sort = true,
      float = {
        border = "rounded",
        source = "always",
      },
    })

    vim.g.rustaceanvim = {
      tools = {
        hover_actions = {
          auto_focus = false,
        },
        test_executor = "neotest",
        inlay_hints = {
          auto = true,
          only_current_line = true, -- Tampilkan hanya di current line
          show_parameter_hints = true,
        },
      },

      server = {
        cmd = function()
          local stable_ra = vim.fn.expand("~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rust-analyzer")
          if vim.fn.executable(stable_ra) == 1 then
            return { stable_ra }
          end
          return { "rust-analyzer" }
        end,
        on_attach = function(_, bufnr)
          local opts = { buffer = bufnr, silent = true }

          if vim.lsp.inlay_hint then
            vim.lsp.inlay_hint.enable(true, { bufnr = bufnr })
          end

          vim.keymap.set("n", "<leader>ih", function()
            vim.lsp.inlay_hint.enable(not vim.lsp.inlay_hint.is_enabled({ bufnr = bufnr }), { bufnr = bufnr })
          end, { buffer = bufnr, desc = "Toggle inlay hints" })

          vim.keymap.set("n", "<leader>rr", "<cmd>RustLsp runnables<CR>", opts)
          vim.keymap.set("n", "<leader>rt", "<cmd>RustLsp testables<CR>", opts)
          vim.keymap.set("n", "<leader>rd", "<cmd>RustLsp debuggables<CR>", opts)
          vim.keymap.set("n", "<leader>re", "<cmd>RustLsp expandMacro<CR>", opts)
          vim.keymap.set("n", "<leader>rc", "<cmd>RustLsp openCargo<CR>", opts)
          vim.keymap.set("n", "<leader>rp", "<cmd>RustLsp parentModule<CR>", opts)
          vim.keymap.set("n", "K", "<cmd>RustLsp hover actions<CR>", opts)

          vim.keymap.set({ "n", "v" }, "<leader>ca", "<cmd>RustLsp codeAction<CR>", opts)

          vim.keymap.set("n", "<leader>qf", function()
            vim.lsp.buf.code_action({
              filter = function(a)
                return a.isPreferred
              end,
              apply = true,
            })
          end, { buffer = bufnr, desc = "Quick fix" })

          vim.keymap.set("n", "]d", vim.diagnostic.goto_next, opts)
          vim.keymap.set("n", "[d", vim.diagnostic.goto_prev, opts)
        end,

        default_settings = {
          ["rust-analyzer"] = {
            cargo = {
              allFeatures = true,
              loadOutDirsFromCheck = true,
              buildScripts = {
                enable = true,
              },
            },

            completion = {
              postfix = {
                enable = true,
              },
            },

            procMacro = {
              enable = true,
            },

            check = {
              command = "check",
            },

            diagnostics = {
              enable = true,
              disabled = { "unresolved-proc-macro" },
            },
            inlayHints = {
              bindingModeHints = { enable = false },
              chainingHints = { enable = true },
              closingBraceHints = {
                enable = true,
                minLines = 25,
              },
              closureReturnTypeHints = { enable = "never" },
              lifetimeElisionHints = {
                enable = "never",
                useParameterNames = false,
              },
              maxLength = 25,
              parameterHints = { enable = true },
              reborrowHints = { enable = "never" },
              renderColons = true,
              typeHints = {
                enable = true, -- ← INI yang nampilin tipe variabel
                hideClosureInitialization = false,
                hideNamedConstructor = false,
              },
            },
          },
        },
      },

      dap = {
        adapter = {
          type = "server",
          port = "${port}",
          executable = {
            command = vim.fn.stdpath("data") .. "/mason/packages/codelldb/extension/adapter/codelldb",
            args = { "--port", "${port}" },
          },
        },
      },
    }
  end,
}
