; 该脚本使用 HM VNISEdit 脚本编辑器向导产生


; 安装程序初始定义常量
!define PRODUCT_NAME $(ToolLang)
!define PRODUCT_VERSION "v1.14"
!define /date DATESTR "%y%m%d"
!define PRODUCT_PUBLISHER "Austin.Young"
!define PRODUCT_WEB_SITE "https://github.com/pattazl/showKeyBoard"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\ShowKeyBoard.exe"
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
;!insertmacro MUI_PAGE_LICENSE "c:\path\to\licence\YourSoftwareLicence.txt"
; 安装目录选择页面
!insertmacro MUI_PAGE_DIRECTORY
; 安装过程页面
!insertmacro MUI_PAGE_INSTFILES
; 安装完成页面
!define MUI_FINISHPAGE_RUN "$INSTDIR\ShowKeyBoard.exe"
!insertmacro MUI_PAGE_FINISH

; 安装卸载过程页面
!insertmacro MUI_UNPAGE_INSTFILES

; 安装界面包含的语言设置
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "SimpChinese"

; 安装预释放文件
!insertmacro MUI_RESERVEFILE_INSTALLOPTIONS
; ------ MUI 现代界面定义结束 ------

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "release\ShowKeyBoard_Setup_${PRODUCT_VERSION}_${DATESTR}.exe"
InstallDir "$PROGRAMFILES\ShowKeyBoard"
InstallDirRegKey HKLM "${PRODUCT_UNINST_KEY}" "UninstallString"
ShowInstDetails show
ShowUnInstDetails show
BrandingText "ShowKeyBoard"


LangString ToolLang ${LANG_SIMPCHINESE} "ShowKeyBoard,键盘鼠标行为分析工具"
LangString ToolLang ${LANG_ENGLISH} "ShowKeyBoard,Statistic for keyboard/mouse"


LangString UNINSTALL_CONFIRM ${LANG_SIMPCHINESE} " 你确定要卸载 "
LangString UNINSTALL_CONFIRM ${LANG_ENGLISH} "Are you sure uninstall !"

LangString UNINSTALL_SUCC ${LANG_SIMPCHINESE} "已成功地从你的计算机移除。"
LangString UNINSTALL_SUCC ${LANG_ENGLISH} "Uninstall success."

LangString KeyBoardPath ${LANG_SIMPCHINESE} "键盘鼠标"
LangString KeyBoardPath ${LANG_ENGLISH} "KeyBoard"

LangString warnmsg1 ${LANG_ENGLISH} "Find the configuration in the installation directory. Do you want to delete it?$\r$\n click $\"YES$\" for delete, click $\"NO$\" reserved"
LangString warnmsg1 ${LANG_SIMPCHINESE} "发现安装目录下原先的配置文件，是否删除？$\r$\n单击“YES”删除，单击“NO”保留"

LangString warnmsg2 ${LANG_ENGLISH} "Find the records in the installation directory. Do you want to delete it?$\r$\n click $\"YES$\" for delete, click $\"NO$\" reserved"
LangString warnmsg2 ${LANG_SIMPCHINESE} "发现安装目录下原先的统计记录，是否删除？$\r$\n单击“YES”删除，单击“NO”保留"

LangString unRegMsg ${LANG_ENGLISH} "Whether reserved relative config or records?$\r$\n$\r$\n"
LangString unRegMsg ${LANG_SIMPCHINESE} "是否保留配置和记录？$\r$\n$\r$\n确定保留？（单击“YES”保留，单击“NO”清除，建议保留）"


Section "MainSection" SEC01

  SetOutPath "$INSTDIR"
  
IfFileExists '$INSTDIR\showKeyBoard.ini' +1 +4
  MessageBox MB_YESNO|MB_DEFBUTTON2 $(warnmsg1) IDNO +3
  delete "$SYSDIR\showKeyBoard.ini"
  delete "$SYSDIR\KeyList.txt"

IfFileExists '$INSTDIR\httpdist\dist\records.db' +1 +3
  MessageBox MB_YESNO|MB_DEFBUTTON2 $(warnmsg2) IDNO +2
  delete "$INSTDIR\httpdist\dist\records.db"
  
; 配置文件如果存在不能覆盖
  SetOverwrite on
    File "ShowKeyBoard.exe"
  SetOverwrite off
    File "showKeyBoard.ini"
    File "KeyList.txt"
  SetOverwrite on	
  SetOutPath "$INSTDIR\httpdist"
    File "httpdist\package.json"
  SetOutPath "$INSTDIR\httpdist\dist"
    File "httpdist\dist\node.exe"
    File "httpdist\dist\server.js"
    File "httpdist\dist\showKeyBoard.desc.ini"
  SetOverwrite off
    File "httpdist\dist\records.db"
  SetOutPath "$INSTDIR\httpdist\dist\lib"
    File /r "httpdist\dist\lib\*"
  SetOverwrite on
  SetOutPath "$INSTDIR\httpdist\dist\ui"
    File /r "httpdist\dist\ui\*"


  ClearErrors

  SetOutPath "$INSTDIR"
  CreateDirectory "$SMPROGRAMS\$(KeyBoardPath)"
  CreateShortCut "$SMPROGRAMS\$(KeyBoardPath)\ShowKeyBoard.lnk" "$INSTDIR\ShowKeyBoard.exe" "" "$INSTDIR\ShowKeyBoard.exe"
  CreateShortCut "$DESKTOP\ShowKeyBoard.lnk" "$INSTDIR\ShowKeyBoard.exe" "" "$INSTDIR\ShowKeyBoard.exe"
SectionEnd

Section -AdditionalIcons
  ;WriteIniStr "$INSTDIR\${PRODUCT_NAME}.url" "InternetShortcut" "URL" "${PRODUCT_WEB_SITE}"
  CreateShortCut "$SMPROGRAMS\$(KeyBoardPath)\Uninstall.lnk" "$INSTDIR\uninst.exe"
SectionEnd

Section -Post
  WriteUninstaller "$INSTDIR\uninst.exe"
  WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\ShowKeyBoard.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "$(^Name)"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninst.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\ShowKeyBoard.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  ;WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"

SectionEnd


/******************************
 *  以下是安装程序的卸载部分  *
 ******************************/

Section Uninstall
    MessageBox MB_YESNO $(unRegMsg) IDYES false IDNO true
    true:
	Delete "$INSTDIR\showKeyBoard.ini"
    Delete "$INSTDIR\KeyList.txt"
    Delete "$INSTDIR\httpdist\dist\records.db"
    
	false:
    Delete "$INSTDIR\uninst.exe"
    Delete "$INSTDIR\showKeyBoard.exe"
    Delete "$SMPROGRAMS\$(KeyBoardPath)\Uninstall.lnk"
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

#-- 根据 NSIS 脚本编辑规则，所有 Function 区段必须放置在 Section 区段之后编写，以避免安装程序出现未可预知的问题。--#
Function .onInit
  !insertmacro MUI_LANGDLL_DISPLAY
FunctionEnd

Function un.onInit
  MessageBox MB_ICONQUESTION|MB_YESNO|MB_DEFBUTTON2 "$(UNINSTALL_CONFIRM) $(^Name) ？" IDYES +2
  Abort
FunctionEnd

Function un.onUninstSuccess
  HideWindow
  MessageBox MB_ICONINFORMATION|MB_OK "$(^Name) $(UNINSTALL_SUCC)"
FunctionEnd
