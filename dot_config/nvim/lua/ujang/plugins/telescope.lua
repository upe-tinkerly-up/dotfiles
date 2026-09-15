return {
  "nvim-telescope/telescope.nvim",
  branch = "0.1.x",
  pin = true,
  dependencies = {
    "nvim-lua/plenary.nvim",
    { "nvim-telescope/telescope-fzf-native.nvim", build = "make" },
    "nvim-tree/nvim-web-devicons",
    "folke/todo-comments.nvim",
    "folke/trouble.nvim", -- Tambahkan dependency
  },
  config = function()
    vim.treesitter.highlighter.is_enabled = function(bufnr)
      return vim.treesitter.highlighter.active[bufnr] ~= nil
    end

    local telescope = require("telescope")
    local actions = require("telescope.actions")
    local transform_mod = require("telescope.actions.mt").transform_mod

    -- Gunakan API baru dari trouble
    local open_with_trouble = require("trouble.sources.telescope").open

    -- Custom action untuk quickfix
    local custom_actions = transform_mod({
      open_trouble_qflist = function(prompt_bufnr)
        require("trouble").toggle("quickfix")
      end,
    })

    telescope.setup({
      defaults = {
        preview = {
          treesitter = false,
        },
        path_display = { "smart" },
        file_ignore_patterns = {
          "node_modules/",
          "%.git/",
        },
        mappings = {
          i = {
            ["<C-k>"] = actions.move_selection_previous, -- move to prev result
            ["<C-j>"] = actions.move_selection_next, -- move to next result
            ["<C-q>"] = actions.send_selected_to_qflist + custom_actions.open_trouble_qflist,
            ["<C-t>"] = open_with_trouble, -- Gunakan fungsi baru
          },
        },
      },
    })

    telescope.load_extension("fzf")

    -- set keymaps
    local keymap = vim.keymap
    keymap.set("n", "<leader>ff", "<cmd>Telescope find_files<cr>", { desc = "Fuzzy find files in cwd" })
    keymap.set("n", "<leader>fr", "<cmd>Telescope oldfiles<cr>", { desc = "Fuzzy find recent files" })
    keymap.set("n", "<leader>fs", "<cmd>Telescope live_grep<cr>", { desc = "Find string in cwd" })
    keymap.set("n", "<leader>fc", "<cmd>Telescope grep_string<cr>", { desc = "Find string under cursor in cwd" })
    keymap.set("n", "<leader>ft", "<cmd>TodoTelescope<cr>", { desc = "Find todos" })
  end,
}
