;编译信息
;@Ahk2Exe-SetName ShowKeyBoard
;@Ahk2Exe-SetDescription Show and Analyse Mouse/KeyBoard
;@Ahk2Exe-SetProductVersion 1.33.0.0
;@Ahk2Exe-SetFileVersion 1.33.0.0
;@Ahk2Exe-SetCopyright Austing.Young (2023 - )
;@Ahk2Exe-SetMainIcon res\keyboard.ico
;@Ahk2Exe-ExeName build/release/ShowKeyBoard.exe
#Requires AutoHotkey v1
#SingleInstance Ignore
global APPName:="ShowKeyBoard", ver:="1.33" 
#include common.ahk
#include langVars.ahk
