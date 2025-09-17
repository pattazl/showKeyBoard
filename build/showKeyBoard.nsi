; 该脚本使用 HM VNISEdit 脚本编辑器向导产生


; 安装程序初始定义常量
!define PRODUCT_NAME $(ToolLang)
!define PRODUCT_VERSION "v1.51"
!define /date DATESTR "%y%m%d"
!define ExeName "showKeyBoard.exe"
!define PRODUCT_PUBLISHER "Austin.Young"
!define PRODUCT_WEB_SITE "https://github.com/pattazl/showKeyBoard"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\${ExeName}"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKLM"

SetCompressor lzma

; ------ MUI 现代界面定义 (1.67 版本以上兼容) ------
!include "MUI.nsh"
;!include "FileFunc.nsh"
;!insertmacro GetTime
;!insertmacro un.GetTime
!include "FileFunc.nsh"
!insertmacro GetTime
; MUI 预定义常量
!define MUI_ABORTWARNING
;!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_ICON "..\res\keyboard.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"
; 语言选择窗口常量设置
!define MUI_LANGDLL_REGISTRY_ROOT "${PRODUCT_UNINST_ROOT_KEY}"
!define MUI_LANGDLL_REGISTRY_KEY "${PRODUCT_UNINST_KEY}"
!define MUI_LANGDLL_REGISTRY_VALUENAME "NSIS:Language"

; 欢迎页面
!insertmacro MUI_PAGE_WELCOME
; 许可协议页面
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
; 组件选择页面
!insertmacro MUI_PAGE_COMPONENTS
; 安装目录选择页面
!insertmacro MUI_PAGE_DIRECTORY
; 安装过程页面
!insertmacro MUI_PAGE_INSTFILES
; 安装完成页面
!define MUI_FINISHPAGE_RUN "$INSTDIR\${ExeName}"
!insertmacro MUI_PAGE_FINISH

; 安装卸载过程页面
!insertmacro MUI_UNPAGE_INSTFILES

; 安装界面包含的语言设置
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "SimpChinese"

; 安装预释放文件
!insertmacro MUI_RESERVEFILE_LANGDLL
!insertmacro MUI_RESERVEFILE_INSTALLOPTIONS
; ------ MUI 现代界面定义结束 ------

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "publish\ShowKeyBoard_Setup_${PRODUCT_VERSION}_${DATESTR}.exe"
InstallDir "$PROGRAMFILES\ShowKeyBoard"
InstallDirRegKey HKLM "${PRODUCT_UNINST_KEY}" "UninstallString"
ShowInstDetails show
ShowUnInstDetails show

; 产品名是作为安装卸载的依据，同时会记录语言选择框
LangString ToolLang ${LANG_SIMPCHINESE} "键盘鼠标行为分析工具"  
LangString ToolLang ${LANG_ENGLISH} "ShowKeyBoard Statistic"

LangString UNINSTALL_SUCC ${LANG_SIMPCHINESE} "已成功地从你的计算机移除。"
LangString UNINSTALL_SUCC ${LANG_ENGLISH} "Uninstall success."

LangString KeyBoardPath ${LANG_SIMPCHINESE} "键盘鼠标"
LangString KeyBoardPath ${LANG_ENGLISH} "KeyboardMouse"

LangString delConfig ${LANG_ENGLISH} "Delete last config"
LangString delConfig ${LANG_SIMPCHINESE} "删除旧配置"

LangString delConfigDesc ${LANG_ENGLISH} "Config include the parameters of client and Server"
LangString delConfigDesc ${LANG_SIMPCHINESE} "配置文件包含客户端和服务端启动的相关信息"

LangString delRecord ${LANG_ENGLISH} "Delete last record"
LangString delRecord ${LANG_SIMPCHINESE} "删除旧统计"
LangString delRecordDesc ${LANG_ENGLISH} "Records for statistic of mouse, keystroke,keymap,Apps"
LangString delRecordDesc ${LANG_SIMPCHINESE} "统计记录包含键盘、鼠标、应用的相关信息"

LangString unRegMsg ${LANG_ENGLISH} "Whether reserved relative config or records?$\r$\n$\r$\n"
LangString unRegMsg ${LANG_SIMPCHINESE} "是否保留配置和记录？$\r$\n$\r$\n确定保留？（单击“YES”保留，单击“NO”清除，建议保留）"

LangString mainSect ${LANG_SIMPCHINESE} "核心模块"
LangString mainSect ${LANG_ENGLISH} "Main"
LangString mainSectDesc ${LANG_SIMPCHINESE} "核心模块，必须安装"
LangString mainSectDesc ${LANG_ENGLISH} "Must be installed"

LangString deskSect ${LANG_SIMPCHINESE} "桌面快捷方式"
LangString deskSect ${LANG_ENGLISH} "Desk shortcut"
LangString deskSectDesc ${LANG_SIMPCHINESE} "创建桌面快捷方式"
LangString deskSectDesc ${LANG_ENGLISH} "Create desk shortcut"

LangString menuSect ${LANG_SIMPCHINESE} "菜单快捷方式"
LangString menuSect ${LANG_ENGLISH} "Menu shortcut"
LangString menuSectDesc ${LANG_SIMPCHINESE} "创建菜单快捷方式"
LangString menuSectDesc ${LANG_ENGLISH} "Create shortcut in windows menu"

LangString Prepare ${LANG_SIMPCHINESE} "准备"
LangString Prepare ${LANG_ENGLISH} "Prepare"

Section $(Prepare) Prepare
  SectionIn RO
SectionEnd
Section /o $(delConfig) delConfig
  ;MessageBox MB_OK "$INSTDIR\showKeyBoard.ini"
  delete "$INSTDIR\showKeyBoard.ini"
  delete "$INSTDIR\KeyList.txt"
SectionEnd
Section /o $(delRecord) delRecord
  ;MessageBox MB_OK "$INSTDIR\httpdist\dist\records.db"
  delete "$INSTDIR\httpdist\dist\records.db"
SectionEnd

Section $(mainSect) mainSect
  SectionIn RO ; 将 section 设置为只读，以禁止用户选择或取消选择

  SetOutPath "$INSTDIR"
  SetOverwrite on
    File "release\${ExeName}"
    File "release\fixStat修复统计数据.bat"

; 配置文件如果存在不能覆盖
  SetOverwrite off
    File "release\showKeyBoard.ini"
    File "release\KeyList.txt"
  SetOutPath "$INSTDIR\httpdist"
  SetOverwrite on
    File "release\httpdist\package.json"
  SetOutPath "$INSTDIR\httpdist\dist"
    File "release\httpdist\dist\node.exe"
    File "release\httpdist\dist\server.js"
    File "release\httpdist\dist\showKeyBoard.desc.ini"
  SetOverwrite off
    File "release\httpdist\dist\records.db"
  SetOutPath "$INSTDIR\httpdist\dist\lib"
    File /r "release\httpdist\dist\lib\*"
  SetOverwrite on
  SetOutPath "$INSTDIR\httpdist\dist\ui"
    RMDir /r "$INSTDIR\httpdist\dist\ui\*"
    File /r "release\httpdist\dist\ui\*"

  ClearErrors

SectionEnd


Section -AdditionalIcons
    WriteIniStr "$INSTDIR\Website.url" "InternetShortcut" "URL" "${PRODUCT_WEB_SITE}"
SectionEnd

Section -Post
  WriteUninstaller "$INSTDIR\uninst.exe"
  WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\${ExeName}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "$(^Name)"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninst.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\${ExeName}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  ;WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"

  ${If} $LANGUAGE == "1033"
      WriteINIStr "$INSTDIR\showKeyBoard.ini" "common" "defaultLang" "en-US"
  ${ElseIf} $LANGUAGE == "2052"
      WriteINIStr "$INSTDIR\showKeyBoard.ini" "common" "defaultLang" "zh-CN"
  ${EndIf}
SectionEnd

Section $(deskSect) deskSect
    ;SetShellVarContext current
    CreateShortCut "$DESKTOP\ShowKeyBoard.lnk" "$INSTDIR\${ExeName}" "" "$INSTDIR\${ExeName}"
SectionEnd

Section $(menuSect) menuSect
    ;SetShellVarContext current
    CreateDirectory "$SMPROGRAMS\$(KeyBoardPath)"
    CreateShortCut "$SMPROGRAMS\$(KeyBoardPath)\ShowKeyBoard.lnk" "$INSTDIR\${ExeName}" "" "$INSTDIR\${ExeName}"
    CreateShortCut "$SMPROGRAMS\$(KeyBoardPath)\Website.lnk" "$INSTDIR\Website.url"
    CreateShortCut "$SMPROGRAMS\$(KeyBoardPath)\Uninstall.lnk" "$INSTDIR\uninst.exe"
SectionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${Prepare} ""
    !insertmacro MUI_DESCRIPTION_TEXT ${mainSect} $(mainSectDesc)
    !insertmacro MUI_DESCRIPTION_TEXT ${deskSect} $(deskSectDesc)
    !insertmacro MUI_DESCRIPTION_TEXT ${menuSect} $(menuSectDesc)
    !insertmacro MUI_DESCRIPTION_TEXT ${delConfig} $(delConfigDesc)
    !insertmacro MUI_DESCRIPTION_TEXT ${delRecord} $(delRecordDesc)

!insertmacro MUI_FUNCTION_DESCRIPTION_END
/******************************
 *  以下是安装程序的卸载部分  *
 ******************************/
Section Uninstall
    Call un.Detect   
    MessageBox MB_YESNO $(unRegMsg) IDYES keepConf IDNO removeConf
    removeConf:
	Delete "$INSTDIR\showKeyBoard.ini"
    Delete "$INSTDIR\KeyList.txt"
    Delete "$INSTDIR\httpdist\dist\records.db"
    RMDir /r "$INSTDIR\backup\*"
    
	keepConf:
    Delete "$INSTDIR\Website.url"
    Delete "$INSTDIR\uninst.exe"
    Delete "$INSTDIR\${ExeName}"
    Delete "$INSTDIR\fixStat修复统计数据.bat"
    Delete "$SMPROGRAMS\$(KeyBoardPath)\Uninstall.lnk"
    Delete "$SMPROGRAMS\$(KeyBoardPath)\Website.lnk"
    Delete "$DESKTOP\ShowKeyBoard.lnk"
    Delete "$SMPROGRAMS\$(KeyBoardPath)\ShowKeyBoard.lnk"

    RMDir "$SMPROGRAMS\$(KeyBoardPath)"
    RMDir /r "$INSTDIR\httpdist\dist\lib\*"
    RMDir /r "$INSTDIR\httpdist\dist\ui\*"
    RMDir /r "$INSTDIR\httpdist\dist\uploads\*"
    Delete "$INSTDIR\httpdist\dist\kbserver.pid"
    Delete "$INSTDIR\httpdist\dist\server.js"
    Delete "$INSTDIR\httpdist\dist\showKeyBoard.desc.ini"
    Delete "$INSTDIR\httpdist\dist\updateTime.txt"
    Delete "$INSTDIR\httpdist\dist\lastRecord.json"
    Delete "$INSTDIR\httpdist\dist\node.exe"
	Delete "$INSTDIR\httpdist\package.json"

    RMDir "$INSTDIR\httpdist\dist\"
    RMDir "$INSTDIR\httpdist\"
    RMDir "$INSTDIR"

    DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"
    DeleteRegKey HKLM "${PRODUCT_DIR_REGKEY}"
    SetAutoClose true
SectionEnd

Var isRunning
Function Detect
  StrCpy $isRunning "安装程序检测到 ${PRODUCT_NAME} 正在运行。$\r$\n$\r$\n点击 “确定” 重试，$\r$\n点击 “取消” 退出当前程序。"
  ${If} $LANGUAGE == "1033"
      StrCpy $isRunning "Detect ${PRODUCT_NAME} is running$\r$\n$\r$\n click $\"YES$\" for retry, click $\"NO$\" abort install/uninstall"
  ${EndIf}
  findRun:
  nsProcess::_FindProcess "${ExeName}"
  Pop $R0
  IntCmp $R0 0 isRun no_run no_run
  isRun:
  MessageBox MB_OKCANCEL|MB_ICONSTOP  $isRunning IDOK findRun IDCANCEL Exit
  ;nsProcess::_KillProcess "AppSetup.exe"
  Exit:
  Quit  ; Quit Abort
  no_run:
FunctionEnd
; 卸载和安装函数无法复用
Function un.Detect
  StrCpy $isRunning "安装程序检测到 ${PRODUCT_NAME} 正在运行。$\r$\n$\r$\n点击 “确定” 重试，$\r$\n点击 “取消” 退出当前程序。"
  ${If} $LANGUAGE == "1033"
      StrCpy $isRunning "Detect ${PRODUCT_NAME} is running$\r$\n$\r$\n click $\"YES$\" for retry, click $\"NO$\" abort install/uninstall"
  ${EndIf}
  findRun:
  nsProcess::_FindProcess "${ExeName}"
  Pop $R0
  IntCmp $R0 0 isRun no_run no_run
  isRun:
  MessageBox MB_OKCANCEL|MB_ICONSTOP  $isRunning IDOK findRun IDCANCEL Exit
  ;nsProcess::_KillProcess "AppSetup.exe"
  Exit:
  Quit  ; Quit Abort
  no_run:
FunctionEnd
#-- 根据 NSIS 脚本编辑规则，所有 Function 区段必须放置在 Section 区段之后编写，以避免安装程序出现未可预知的问题。--#
Function .onInit
  !insertmacro MUI_LANGDLL_DISPLAY
  ReadRegStr $0 HKLM "${PRODUCT_DIR_REGKEY}" ""
  ; 如果找到安装目录，则设置为默认安装目录
  ${If} $0 != ""
    ${GetParent} $0 $R0
    StrCpy $INSTDIR $R0
  ${EndIf}
  Call Detect
FunctionEnd

Var UNINSTALL_CONFIRM
Function un.onInit
; $LANGUAGE  从配置文件中读取语言
  StrCpy $UNINSTALL_CONFIRM " 你确定要卸载 "
  ReadINIStr $0 "$INSTDIR\showKeyBoard.ini" "common" "defaultLang"
  ${If} $0 == "en-US" ; 英文
    StrCpy $LANGUAGE "1033"
    StrCpy $UNINSTALL_CONFIRM "Are you sure uninstall !"
  ${EndIf}
  ;!insertmacro MUI_UNGETLANGUAGE
  MessageBox MB_ICONQUESTION|MB_YESNO|MB_DEFBUTTON2 "$UNINSTALL_CONFIRM $(^Name) ？" IDYES +2
  Abort
FunctionEnd

Function un.onUninstSuccess
  HideWindow
  MessageBox MB_ICONINFORMATION|MB_OK "$(^Name) $(UNINSTALL_SUCC)"
FunctionEnd
