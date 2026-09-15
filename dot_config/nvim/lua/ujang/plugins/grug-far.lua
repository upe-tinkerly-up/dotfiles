return {
  "MagicDuck/grug-far.nvim",
  config = function()
    require("grug-far").setup({
      -- Konfigurasi default sudah bagus
    })
  end,
  keys = {
    {
      "<leader>rr",
      function()
        require("grug-far").open()
      end,
      desc = "Search and Replace (Grug Far)",
    },
    {
      "<leader>rw",
      function()
        require("grug-far").open({ prefills = { search = vim.fn.expand("<cword>") } })
      end,
      desc = "Replace word under cursor",
    },
    {
      "<leader>rf",
      function()
        require("grug-far").open({ prefills = { paths = vim.fn.expand("%") } })
      end,
      desc = "Replace in current file",
    },
  },
}
