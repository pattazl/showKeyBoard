; �ýű�ʹ�� HM VNISEdit �ű��༭���򵼲���


; ��װ�����ʼ���峣��
!define PRODUCT_NAME $(ToolLang)
!define PRODUCT_VERSION "v1.14"
!define /date DATESTR "%y%m%d"
!define PRODUCT_PUBLISHER "Austin.Young"
!define PRODUCT_WEB_SITE "https://github.com/pattazl/showKeyBoard"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\ShowKeyBoard.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKLM"

SetCompressor lzma

; ------ MUI �ִ����涨�� (1.67 �汾���ϼ���) ------
!include "MUI.nsh"
;!include "FileFunc.nsh"
;!insertmacro GetTime
;!insertmacro un.GetTime
!include "FileFunc.nsh"
!insertmacro GetTime
; MUI Ԥ���峣��
!define MUI_ABORTWARNING
;!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_ICON "..\res\keyboard.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"
; ����ѡ�񴰿ڳ�������
!define MUI_LANGDLL_REGISTRY_ROOT "${PRODUCT_UNINST_ROOT_KEY}"
!define MUI_LANGDLL_REGISTRY_KEY "${PRODUCT_UNINST_KEY}"
!define MUI_LANGDLL_REGISTRY_VALUENAME "NSIS:Language"

; ��ӭҳ��
!insertmacro MUI_PAGE_WELCOME
; ���Э��ҳ��
;!insertmacro MUI_PAGE_LICENSE "c:\path\to\licence\YourSoftwareLicence.txt"
; ��װĿ¼ѡ��ҳ��
!insertmacro MUI_PAGE_DIRECTORY
; ��װ����ҳ��
!insertmacro MUI_PAGE_INSTFILES
; ��װ���ҳ��
!define MUI_FINISHPAGE_RUN "$INSTDIR\ShowKeyBoard.exe"
!insertmacro MUI_PAGE_FINISH

; ��װж�ع���ҳ��
!insertmacro MUI_UNPAGE_INSTFILES

; ��װ�����������������
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "SimpChinese"

; ��װԤ�ͷ��ļ�
!insertmacro MUI_RESERVEFILE_INSTALLOPTIONS
; ------ MUI �ִ����涨����� ------

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "release\ShowKeyBoard_Setup_${PRODUCT_VERSION}_${DATESTR}.exe"
InstallDir "$PROGRAMFILES\ShowKeyBoard"
InstallDirRegKey HKLM "${PRODUCT_UNINST_KEY}" "UninstallString"
ShowInstDetails show
ShowUnInstDetails show
BrandingText "ShowKeyBoard"


LangString ToolLang ${LANG_SIMPCHINESE} "ShowKeyBoard,���������Ϊ��������"
LangString ToolLang ${LANG_ENGLISH} "ShowKeyBoard,Statistic for keyboard/mouse"


LangString UNINSTALL_CONFIRM ${LANG_SIMPCHINESE} " ��ȷ��Ҫж�� "
LangString UNINSTALL_CONFIRM ${LANG_ENGLISH} "Are you sure uninstall !"

LangString UNINSTALL_SUCC ${LANG_SIMPCHINESE} "�ѳɹ��ش���ļ�����Ƴ���"
LangString UNINSTALL_SUCC ${LANG_ENGLISH} "Uninstall success."

LangString KeyBoardPath ${LANG_SIMPCHINESE} "�������"
LangString KeyBoardPath ${LANG_ENGLISH} "KeyBoard"

LangString warnmsg1 ${LANG_ENGLISH} "Find the configuration in the installation directory. Do you want to delete it?$\r$\n click $\"YES$\" for delete, click $\"NO$\" reserved"
LangString warnmsg1 ${LANG_SIMPCHINESE} "���ְ�װĿ¼��ԭ�ȵ������ļ����Ƿ�ɾ����$\r$\n������YES��ɾ����������NO������"

LangString warnmsg2 ${LANG_ENGLISH} "Find the records in the installation directory. Do you want to delete it?$\r$\n click $\"YES$\" for delete, click $\"NO$\" reserved"
LangString warnmsg2 ${LANG_SIMPCHINESE} "���ְ�װĿ¼��ԭ�ȵ�ͳ�Ƽ�¼���Ƿ�ɾ����$\r$\n������YES��ɾ����������NO������"

LangString unRegMsg ${LANG_ENGLISH} "Whether reserved relative config or records?$\r$\n$\r$\n"
LangString unRegMsg ${LANG_SIMPCHINESE} "�Ƿ������úͼ�¼��$\r$\n$\r$\nȷ����������������YES��������������NO����������鱣����"


Section "MainSection" SEC01

  SetOutPath "$INSTDIR"
  
IfFileExists '$INSTDIR\showKeyBoard.ini' +1 +4
  MessageBox MB_YESNO|MB_DEFBUTTON2 $(warnmsg1) IDNO +3
  delete "$SYSDIR\showKeyBoard.ini"
  delete "$SYSDIR\KeyList.txt"

IfFileExists '$INSTDIR\httpdist\dist\records.db' +1 +3
  MessageBox MB_YESNO|MB_DEFBUTTON2 $(warnmsg2) IDNO +2
  delete "$INSTDIR\httpdist\dist\records.db"
  
; �����ļ�������ڲ��ܸ���
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
 *  �����ǰ�װ�����ж�ز���  *
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

#-- ���� NSIS �ű��༭�������� Function ���α�������� Section ����֮���д���Ա��ⰲװ�������δ��Ԥ֪�����⡣--#
Function .onInit
  !insertmacro MUI_LANGDLL_DISPLAY
FunctionEnd

Function un.onInit
  MessageBox MB_ICONQUESTION|MB_YESNO|MB_DEFBUTTON2 "$(UNINSTALL_CONFIRM) $(^Name) ��" IDYES +2
  Abort
FunctionEnd

Function un.onUninstSuccess
  HideWindow
  MessageBox MB_ICONINFORMATION|MB_OK "$(^Name) $(UNINSTALL_SUCC)"
FunctionEnd
