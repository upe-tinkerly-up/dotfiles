return {
  "numToStr/Comment.nvim",
  event = { "BufReadPre", "BufNewFile" },
  dependencies = {
    {
      "JoosepAlviste/nvim-ts-context-commentstring",
      lazy = true,
      opts = {
        enable_autocmd = false,
      },
    },
  },
  config = function()
    vim.g.skip_ts_context_commentstring_module = true
    require("ts_context_commentstring").setup({ enable_autocmd = false })

    local ts_context_commentstring = require("ts_context_commentstring.integrations.comment_nvim")

    require("Comment").setup({
      pre_hook = ts_context_commentstring.create_pre_hook(),
    })
  end,
}
