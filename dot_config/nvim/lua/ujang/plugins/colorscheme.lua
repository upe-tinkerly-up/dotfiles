return {
  "folke/tokyonight.nvim",
  priority = 1000,
  config = function()
    local transparent = false

    -- Mayukai Mirage: Background & UI
    local bg = "#1F2430"
    local bg_dark = "#1A1F2E"
    local bg_highlight = "#2A3040"
    local bg_search = "#4D5566"
    local bg_visual = "#2D3B50"
    local fg = "#CBCCC6"
    local fg_dark = "#A2A8B8"
    local fg_gutter = "#4D5566"
    local border = "#3D5266"

    require("tokyonight").setup({
      style = "night",
      transparent = transparent,
      styles = {
        sidebars = transparent and "transparent" or "dark",
        floats = transparent and "transparent" or "dark",
        -- Syntax styles
        comments = { italic = true },
        keywords = { italic = true },
        functions = {},
        variables = {},
      },
      on_colors = function(colors)
        -- UI colors
        colors.bg = bg
        colors.bg_dark = transparent and colors.none or bg_dark
        colors.bg_float = transparent and colors.none or bg_dark
        colors.bg_highlight = bg_highlight
        colors.bg_popup = bg_dark
        colors.bg_search = bg_search
        colors.bg_sidebar = transparent and colors.none or bg_dark
        colors.bg_statusline = transparent and colors.none or bg_dark
        colors.bg_visual = bg_visual
        colors.border = border
        colors.fg = fg
        colors.fg_dark = fg_dark
        colors.fg_float = fg
        colors.fg_gutter = fg_gutter
        colors.fg_sidebar = fg_dark

        -- Syntax / token colors (Mayukai Mirage palette)
        colors.yellow = "#FFD580" -- string, attribute
        colors.orange = "#FFAE57" -- keyword, operator
        colors.red = "#F28779" -- error, deleted
        colors.green = "#BAE67E" -- string escape, diff add
        colors.teal = "#95E6CB" -- builtin, special
        colors.cyan = "#5CCFE6" -- type, class
        colors.blue = "#73D0FF" -- function, method
        colors.blue1 = "#73D0FF"
        colors.blue2 = "#5CCFE6"
        colors.blue5 = "#73D0FF"
        colors.blue6 = "#5CCFE6"
        colors.blue7 = "#2D3B50"
        colors.purple = "#D4BFFF" -- variable, parameter
        colors.magenta = "#F29E74" -- constant, number
        colors.magenta2 = "#F28779"
        colors.comment = "#5C6773" -- comment
        colors.dark3 = "#4D5566"
        colors.dark5 = "#707A8C"
      end,

      on_highlights = function(hl, colors)
        -- Fine-tune specific highlight groups
        hl["@variable"] = { fg = colors.purple }
        hl["@variable.builtin"] = { fg = colors.teal }
        hl["@variable.parameter"] = { fg = colors.purple }
        hl["@function"] = { fg = colors.blue }
        hl["@function.builtin"] = { fg = colors.teal }
        hl["@function.call"] = { fg = colors.blue }
        hl["@method"] = { fg = colors.blue }
        hl["@method.call"] = { fg = colors.blue }
        hl["@keyword"] = { fg = colors.orange, italic = true }
        hl["@keyword.function"] = { fg = colors.orange, italic = true }
        hl["@keyword.return"] = { fg = colors.orange, italic = true }
        hl["@conditional"] = { fg = colors.orange, italic = true }
        hl["@repeat"] = { fg = colors.orange, italic = true }
        hl["@string"] = { fg = colors.yellow }
        hl["@string.escape"] = { fg = colors.green }
        hl["@number"] = { fg = colors.magenta }
        hl["@float"] = { fg = colors.magenta }
        hl["@boolean"] = { fg = colors.magenta }
        hl["@type"] = { fg = colors.cyan }
        hl["@type.builtin"] = { fg = colors.cyan }
        hl["@constant"] = { fg = colors.magenta }
        hl["@constant.builtin"] = { fg = colors.teal }
        hl["@property"] = { fg = fg }
        hl["@field"] = { fg = fg }
        hl["@operator"] = { fg = colors.orange }
        hl["@punctuation"] = { fg = colors.orange }
        hl["@comment"] = { fg = colors.comment, italic = true }
        hl["@tag"] = { fg = colors.orange }
        hl["@tag.attribute"] = { fg = colors.yellow }
        hl["@tag.delimiter"] = { fg = colors.orange }
        -- LSP
        hl["DiagnosticError"] = { fg = colors.red }
        hl["DiagnosticWarn"] = { fg = colors.orange }
        hl["DiagnosticInfo"] = { fg = colors.cyan }
        hl["DiagnosticHint"] = { fg = colors.teal }

        -- JSX component names (identifier dalam jsx context)
        hl["@tag"] = { fg = "#FF8F40" } -- <div>, <span>
        hl["@tag.jsx"] = { fg = "#FF8F40" }
        hl["@tag.builtin.tsx"] = { fg = "#5CCFE6" }
        hl["@tag.attribute"] = { fg = "#FFD580" }
        hl["@tag.attribute.tsx"] = { fg = "#FFD580" }
        hl["@tag.delimiter"] = { fg = "#5C6773" }
        hl["@tag.delimiter.tsx"] = { fg = "#5C6773" }

        -- Ini yang kemungkinan dipakai untuk Typography, Box, dll
        hl["@constructor.tsx"] = { fg = "#5CCFE6" }
        hl["@constructor.jsx"] = { fg = "#5CCFE6" }

        -- LspInlayHint
        hl["LspInlayHint"] = { fg = "#C8A96E", italic = true }
      end,
    })
    vim.cmd("colorscheme tokyonight")
  end,
}
