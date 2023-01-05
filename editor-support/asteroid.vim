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

syn keyword basic and
syn keyword basic as
syn keyword basic assert
syn keyword basic bind
syn keyword basic break
syn keyword basic catch
syn keyword basic data
syn keyword basic do
syn keyword basic elif
syn keyword basic else
syn keyword basic end
syn keyword basic escape
syn keyword basic eval
syn keyword basic for
syn keyword basic from
syn keyword basic function
syn keyword basic global
syn keyword basic if
syn keyword basic in
syn keyword basic is
syn keyword basic lambda
syn keyword basic let
syn keyword basic load
syn keyword basic loop
syn keyword basic not
syn keyword basic or
syn keyword basic orwith
syn keyword basic pattern
syn keyword basic repeat
syn keyword basic return
syn keyword basic step
syn keyword basic structure
syn keyword basic system
syn keyword basic throw
syn keyword basic to
syn keyword basic try
syn keyword basic until
syn keyword basic while
syn keyword basic with

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
