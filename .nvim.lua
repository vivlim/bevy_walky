local term = require("bufterm.terminal")
local ui = require("bufterm.ui")

-- Overload 'make' shortcut since it isn't useful in this repo.
local floating_term = term.Terminal:new({
	cmd = "cargo run --features bevy/dynamic_linking",
	auto_close = true,
	fallback_on_exit = true,
})

vim.keymap.set({ "n" }, "<space>M", function()
	floating_term:spawn()
	ui.toggle_float(floating_term.bufnr)
	vim.api.nvim_set_option_value("number", false, { buf = floating_term.bufnr })
end, {
	desc = "run bevy",
})
