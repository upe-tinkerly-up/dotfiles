if vim.treesitter.language and not vim.treesitter.language.ft_to_lang then
  vim.treesitter.language.ft_to_lang = function(ft)
    local ok, lang = pcall(vim.treesitter.language.get_lang, ft)
    return ok and lang or ft
  end
end

local hl = vim.treesitter.highlighter
if not hl.is_enabled then
  hl.is_enabled = function(bufnr)
    return hl.active[bufnr] ~= nil
  end
end

require("ujang.core")
require("ujang.lazy")
require("ujang.lsp")
