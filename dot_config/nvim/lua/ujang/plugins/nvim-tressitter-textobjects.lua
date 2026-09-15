return {
  "nvim-treesitter/nvim-treesitter-textobjects",
  branch = "main",
  dependencies = { "nvim-treesitter/nvim-treesitter" },
  event = { "BufReadPre", "BufNewFile" },
  config = function()
    local select = require("nvim-treesitter-textobjects.select")
    local move = require("nvim-treesitter-textobjects.move")
    local swap = require("nvim-treesitter-textobjects.swap")

    -- SELECT
    local sel_maps = {
      ["a="] = { "@assignment.outer", "Select outer part of an assignment" },
      ["i="] = { "@assignment.inner", "Select inner part of an assignment" },
      ["l="] = { "@assignment.lhs", "Select left hand side of an assignment" },
      ["r="] = { "@assignment.rhs", "Select right hand side of an assignment" },
      ["a:"] = { "@property.outer", "Select outer part of an object property" },
      ["i:"] = { "@property.inner", "Select inner part of an object property" },
      ["l:"] = { "@property.lhs", "Select left part of an object property" },
      ["r:"] = { "@property.rhs", "Select right part of an object property" },
      ["aa"] = { "@parameter.outer", "Select outer part of a parameter/argument" },
      ["ia"] = { "@parameter.inner", "Select inner part of a parameter/argument" },
      ["ai"] = { "@conditional.outer", "Select outer part of a conditional" },
      ["ii"] = { "@conditional.inner", "Select inner part of a conditional" },
      ["al"] = { "@loop.outer", "Select outer part of a loop" },
      ["il"] = { "@loop.inner", "Select inner part of a loop" },
      ["af"] = { "@call.outer", "Select outer part of a function call" },
      ["if"] = { "@call.inner", "Select inner part of a function call" },
      ["am"] = { "@function.outer", "Select outer part of a method/function def" },
      ["im"] = { "@function.inner", "Select inner part of a method/function def" },
      ["ac"] = { "@class.outer", "Select outer part of a class" },
      ["ic"] = { "@class.inner", "Select inner part of a class" },
    }
    for key, val in pairs(sel_maps) do
      vim.keymap.set({ "x", "o" }, key, function()
        select.select_textobject(val[1], "textobjects")
      end, { desc = val[2] })
    end

    -- SWAP NEXT
    local swap_next = {
      ["<leader>na"] = "@parameter.inner",
      ["<leader>n:"] = "@property.outer",
      ["<leader>nm"] = "@function.outer",
    }
    for key, query in pairs(swap_next) do
      vim.keymap.set("n", key, function()
        swap.swap_next(query, "textobjects")
      end)
    end

    -- SWAP PREVIOUS
    local swap_prev = {
      ["<leader>pa"] = "@parameter.inner",
      ["<leader>p:"] = "@property.outer",
      ["<leader>pm"] = "@function.outer",
    }
    for key, query in pairs(swap_prev) do
      vim.keymap.set("n", key, function()
        swap.swap_previous(query, "textobjects")
      end)
    end

    -- MOVE: goto_next_start
    local next_start = {
      ["]f"] = { "@call.outer", "Next function call start" },
      ["]m"] = { "@function.outer", "Next method/function def start" },
      ["]c"] = { "@class.outer", "Next class start" },
      ["]i"] = { "@conditional.outer", "Next conditional start" },
      ["]l"] = { "@loop.outer", "Next loop start" },
      ["]s"] = { "@local.scope", "Next scope" }, -- locals group
      ["]z"] = { "@fold", "Next fold" }, -- folds group
    }
    for key, val in pairs(next_start) do
      local group = (key == "]s") and "locals" or (key == "]z") and "folds" or "textobjects"
      vim.keymap.set({ "n", "x", "o" }, key, function()
        move.goto_next_start(val[1], group)
      end, { desc = val[2] })
    end

    -- MOVE: goto_next_end
    local next_end = {
      ["]F"] = { "@call.outer", "Next function call end" },
      ["]M"] = { "@function.outer", "Next method/function def end" },
      ["]C"] = { "@class.outer", "Next class end" },
      ["]I"] = { "@conditional.outer", "Next conditional end" },
      ["]L"] = { "@loop.outer", "Next loop end" },
    }
    for key, val in pairs(next_end) do
      vim.keymap.set({ "n", "x", "o" }, key, function()
        move.goto_next_end(val[1], "textobjects")
      end, { desc = val[2] })
    end

    -- MOVE: goto_previous_start
    local prev_start = {
      ["[f"] = { "@call.outer", "Prev function call start" },
      ["[m"] = { "@function.outer", "Prev method/function def start" },
      ["[c"] = { "@class.outer", "Prev class start" },
      ["[i"] = { "@conditional.outer", "Prev conditional start" },
      ["[l"] = { "@loop.outer", "Prev loop start" },
    }
    for key, val in pairs(prev_start) do
      vim.keymap.set({ "n", "x", "o" }, key, function()
        move.goto_previous_start(val[1], "textobjects")
      end, { desc = val[2] })
    end

    -- MOVE: goto_previous_end
    local prev_end = {
      ["[F"] = { "@call.outer", "Prev function call end" },
      ["[M"] = { "@function.outer", "Prev method/function def end" },
      ["[C"] = { "@class.outer", "Prev class end" },
      ["[I"] = { "@conditional.outer", "Prev conditional end" },
      ["[L"] = { "@loop.outer", "Prev loop end" },
    }
    for key, val in pairs(prev_end) do
      vim.keymap.set({ "n", "x", "o" }, key, function()
        move.goto_previous_end(val[1], "textobjects")
      end, { desc = val[2] })
    end
  end,
}
