return {
  {
    "nvim-neotest/neotest",
    event = { "BufReadPre", "BufNewFile" },
    dependencies = {
      "nvim-neotest/nvim-nio",
      "nvim-lua/plenary.nvim",
      "antoinemadec/FixCursorHold.nvim",
      "nvim-treesitter/nvim-treesitter",
      -- Adapters
      "rouge8/neotest-rust", -- Rust adapter
      "fredrikaverpil/neotest-golang", -- Go adapter
    },
    config = function()
      local neotest = require("neotest")
      vim.g.neotest_rust = {
        timeout = 15000,
      }

      neotest.setup({
        adapters = {
          -- Rust: pakai `cargo test` di balik layar
          require("neotest-rust")({
            args = { "--nocapture" }, -- lihat stdout saat test
            runner = "cargo",
            dap_adapter = "codelldb",
            timeout = 15000,
          }),

          -- Go: pakai `go test` di balik layar
          require("neotest-golang")({
            go_test_args = { "-v", "-race", "-count=1" },
            dap_go_enabled = false, -- aktifkan jika pakai nvim-dap
          }),
        },

        -- Tampilan output
        output = {
          enabled = true,
          open_on_run = true,
        },

        -- Panel summary di samping
        summary = {
          enabled = true,
          animated = true,
          follow = true,
          expand_errors = true,
        },

        -- Tanda di sign column
        icons = {
          passed = "✓",
          running = "↻",
          failed = "✗",
          unknown = "?",
          skipped = "⊘",
        },

        status = {
          enabled = true,
          virtual_text = true,
          signs = true,
        },

        discovery = {
          enabled = false,
        },
      })

      -- Keymaps
      local keymap = vim.keymap
      keymap.set("n", "<leader>tt", function()
        neotest.run.run()
      end, { desc = "Run nearest test" })

      keymap.set("n", "<leader>tf", function()
        neotest.run.run(vim.fn.expand("%"))
      end, { desc = "Run all tests in file" })

      keymap.set("n", "<leader>tl", function()
        neotest.run.run_last()
      end, { desc = "Run last test" })

      keymap.set("n", "<leader>ts", function()
        neotest.summary.toggle()
      end, { desc = "Toggle test summary panel" })

      keymap.set("n", "<leader>to", function()
        neotest.output.open({ enter = true, auto_close = true })
      end, { desc = "Show test output" })

      keymap.set("n", "<leader>tO", function()
        neotest.output_panel.toggle()
      end, { desc = "Toggle test output panel" })

      keymap.set("n", "<leader>tS", function()
        neotest.run.stop()
      end, { desc = "Stop running tests" })

      -- Navigasi antar test yang gagal
      keymap.set("n", "]n", function()
        neotest.jump.next({ status = "failed" })
      end, { desc = "Jump to next failed test" })

      keymap.set("n", "[n", function()
        neotest.jump.prev({ status = "failed" })
      end, { desc = "Jump to prev failed test" })
    end,
  },
}
