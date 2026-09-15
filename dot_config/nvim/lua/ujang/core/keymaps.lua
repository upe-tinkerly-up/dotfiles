vim.g.mapleader = " "
local keymap = vim.keymap

keymap.set("i", "jk", "<ESC>", { desc = "Exit insert mode with jk" })
keymap.set("n", "<leader>nh", ":nohl<CR>", { desc = "Clear search highlights" })

-- increment/decrement
keymap.set("n", "<leader>+", "<C-a>", { desc = "Increment number" })
keymap.set("n", "<leader>-", "<C-x>", { desc = "Decrement number" })

-- window management
keymap.set("n", "<leader>sv", "<C-w>v", { desc = "Split window vertically" })
keymap.set("n", "<leader>sh", "<C-w>s", { desc = "Split window horizontally" })
keymap.set("n", "<leader>se", "<C-w>=", { desc = "Make splits equal size" })
keymap.set("n", "<leader>sx", "<cmd>close<CR>", { desc = "Close current split" })

-- tab management (ganti prefix ke <leader>b)
keymap.set("n", "<leader>bo", "<cmd>tabnew<CR>", { desc = "Open new tab" })
keymap.set("n", "<leader>bx", "<cmd>tabclose<CR>", { desc = "Close current tab" })
keymap.set("n", "<leader>bn", "<cmd>tabn<CR>", { desc = "Go to next tab" })
keymap.set("n", "<leader>bp", "<cmd>tabp<CR>", { desc = "Go to previous tab" })
keymap.set("n", "<leader>bf", "<cmd>tabnew %<CR>", { desc = "Open current buffer in new tab" })
