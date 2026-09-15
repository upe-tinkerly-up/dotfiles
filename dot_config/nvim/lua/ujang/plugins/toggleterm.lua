return {
  "akinsho/toggleterm.nvim",
  version = "*",
  event = "VeryLazy",
  config = function()
    local toggleterm = require("toggleterm")
    toggleterm.setup({
      size = function(term)
        if term.direction == "horizontal" then
          return 15
        elseif term.direction == "vertical" then
          return vim.o.columns * 0.4
        end
      end,
      open_mapping = [[<C-_>]],
      hide_numbers = true,
      shade_terminals = true,
      shading_factor = 2,
      start_in_insert = true,
      insert_mappings = true,
      terminal_mappings = true,
      persist_size = true,
      direction = "float",
      close_on_exit = true,
      shell = vim.o.shell,
      auto_scroll = true,
      float_opts = {
        border = "curved",
        winblend = 0,
        highlights = {
          border = "Normal",
          background = "Normal",
        },
      },
    })

    local keymap = vim.keymap

    -- Exit terminal mode
    keymap.set("t", "<Esc>", "<C-\\><C-n>", { desc = "Exit terminal mode" })
    keymap.set("t", "jk", "<C-\\><C-n>", { desc = "Exit terminal mode with jk" })

    -- Fallback Ctrl+/
    keymap.set("n", "<C-_>", "<cmd>ToggleTerm<CR>", { desc = "Toggle terminal" })
    keymap.set("t", "<C-_>", "<cmd>ToggleTerm<CR>", { desc = "Toggle terminal" })
    keymap.set("i", "<C-_>", "<cmd>ToggleTerm<CR>", { desc = "Toggle terminal" })

    -- Toggle arah (prefix <leader>tm)
    keymap.set("n", "<leader>tmf", "<cmd>ToggleTerm direction=float<CR>", { desc = "Terminal float" })
    keymap.set("n", "<leader>tmh", "<cmd>ToggleTerm direction=horizontal<CR>", { desc = "Terminal horizontal" })
    keymap.set("n", "<leader>tmv", "<cmd>ToggleTerm direction=vertical<CR>", { desc = "Terminal vertical" })
    keymap.set("n", "<leader>tmt", "<cmd>ToggleTerm direction=tab<CR>", { desc = "Terminal tab" })

    -- Multiple terminal
    keymap.set("n", "<leader>tm1", "<cmd>1ToggleTerm direction=float<CR>", { desc = "Terminal 1" })
    keymap.set("n", "<leader>tm2", "<cmd>2ToggleTerm direction=float<CR>", { desc = "Terminal 2" })
    keymap.set("n", "<leader>tm3", "<cmd>3ToggleTerm direction=float<CR>", { desc = "Terminal 3" })

    -- Lazygit
    local Terminal = require("toggleterm.terminal").Terminal
    local lazygit = Terminal:new({
      cmd = "lazygit",
      dir = "git_dir",
      direction = "float",
      float_opts = { border = "curved" },
      on_open = function(term)
        vim.cmd("startinsert!")
        vim.api.nvim_buf_set_keymap(term.bufnr, "t", "<Esc>", "<Esc>", { noremap = true, silent = true })
      end,
      on_close = function()
        vim.cmd("startinsert!")
      end,
    })

    keymap.set("n", "<leader>lg", function()
      lazygit:toggle()
    end, { desc = "Toggle lazygit" })
  end,
}
