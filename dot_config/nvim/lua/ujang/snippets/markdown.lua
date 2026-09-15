local ls = require("luasnip")
local s = ls.snippet
local t = ls.text_node
local i = ls.insert_node

return {
  s("fm", {
    t({ "---", "title: " }),
    i(1, "Title"),
    t({ "", "date: " }),
    i(2, os.date("%Y-%m-%d")),
    t({ "", "---", "" }),
  }),
  s("cb", {
    t("```"),
    i(1, "lang"),
    t({ "", "" }),
    i(2),
    t({ "", "```" }),
  }),
}
