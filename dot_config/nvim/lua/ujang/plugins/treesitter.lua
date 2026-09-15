return {
  "nvim-treesitter/nvim-treesitter",
  event = { "BufReadPre", "BufNewFile" },
  build = ":TSUpdate",
  config = function()
    local ensure_installed = {
      "json",
      "javascript",
      "typescript",
      "tsx",
      "yaml",
      "html",
      "css",
      "prisma",
      "markdown",
      "markdown_inline",
      "svelte",
      "graphql",
      "bash",
      "lua",
      "vim",
      "dockerfile",
      "gitignore",
      "query",
      "vimdoc",
      "c",
      "rust",
      "toml",
      "go",
      "gomod",
      "gosum",
    }

    require("nvim-treesitter.config").setup({
      highlight = { enable = true },
      indent = { enable = true },
    })

    -- install daftar bahasa yang udah ditentuin
    require("nvim-treesitter").install(ensure_installed)

    -- AUTO INSTALL: kalau buka filetype yang parsernya belum ada, install otomatis
    vim.api.nvim_create_autocmd("FileType", {
      callback = function(args)
        local lang = vim.treesitter.language.get_lang(args.match) or args.match
        local installed = require("nvim-treesitter.config").get_installed()

        if vim.tbl_contains(installed, lang) then
          return
        end

        -- cek apakah parser tersedia buat bahasa ini sebelum nyoba install
        local ok = pcall(vim.treesitter.language.add, lang)
        if not ok then
          require("nvim-treesitter").install({ lang })
        end
      end,
    })

    vim.treesitter.language.register("bash", "zsh")
  end,
}
