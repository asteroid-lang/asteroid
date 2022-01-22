" Vim syntax file
" Language: Asteroid
" Maintainer: Theo Henson <theodorehenson at protonmail dot com>
" Last Change: 2022 Jan 20

" Copy this file to ~/.vim/syntax/asteroid.vim
" For filetype detection create ~/.vim/ftdetect/asteroid.vim containing:
" au BufRead,BufNewFile *.ast set filetype=asteroid

if exists("b:current_syntax")
	finish
endif

syn keyword basic with end do load let for function structure in is pattern 
syn keyword basic throw this system data global return to step if not
syn keyword basic else or and

syn keyword delimeter escape

syn match pattern /%[a-zA-Z]*/
syn match pattern /@[a-z_A-Z]*/
syn match pattern / if /

syn region string start=/\v"/ skip=/\v\\./ end=/\v"/
syn keyword boolean true false
syn match number '\d\+'
syn match number '\d\+\.?\d*'

syn match delimeter '*'
syn match delimeter '\n'

syn match comment '\--.*$'
syn keyword todo TODO FIX FIXME NOTE Note

hi def link basic Statement
hi def link boolean Constant
hi def link number Constant
hi def link pattern Function
hi def link string Constant
hi def link delimeter PreProc
hi def link comment Comment
hi def link todo Todo
