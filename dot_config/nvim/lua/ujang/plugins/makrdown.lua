return {
  -- Render markdown langsung di buffer (heading, checkbox, code block jadi cantik tanpa preview browser)
  {
    "MeanderingProgrammer/render-markdown.nvim",
    ft = { "markdown" },
    dependencies = { "nvim-treesitter/nvim-treesitter", "nvim-tree/nvim-web-devicons" },
    config = function()
      require("render-markdown").setup({
        heading = {
          sign = false,
          icons = { "󰲡 ", "󰲣 ", "󰲥 ", "󰲧 ", "󰲩 ", "󰲫 " },
        },
        checkbox = {
          unchecked = { icon = "󰄱 " },
          checked = { icon = "󰱒 " },
        },
        code = { style = "full" },
      })

      vim.keymap.set("n", "<leader>mr", "<cmd>RenderMarkdown toggle<CR>", { desc = "Toggle Render Markdown" })
    end,
  },

  -- Preview asli di browser (GFM-accurate, bagus buat cek sebelum push ke GitHub/README)

  {
    "iamcco/markdown-preview.nvim",
    cmd = { "MarkdownPreviewToggle", "MarkdownPreview", "MarkdownPreviewStop" },
    ft = { "markdown" },
    build = "cd app && bun install",
    config = function()
      vim.g.mkdp_auto_close = false
      vim.g.mkdp_theme = "dark"

      vim.keymap.set("n", "<leader>mp", "<cmd>MarkdownPreviewToggle<CR>", { desc = "Toggle Markdown Preview" })
    end,
  },

  -- Rapikan tabel otomatis: ketik | terus tab, alignment otomatis
  {
    "dhruvasagar/vim-table-mode",
    ft = { "markdown" },
    config = function()
      vim.g.table_mode_corner = "|" -- biar sesuai GFM style

      vim.keymap.set("n", "<leader>mt", "<cmd>TableModeToggle<CR>", { desc = "Toggle Table Mode" })
    end,
  },
}
