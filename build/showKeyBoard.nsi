; �ýű�ʹ�� HM VNISEdit �ű��༭���򵼲���


; ��װ�����ʼ���峣��
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
!insertmacro MUI_PAGE_LICENSE "..\LICENSE"
; ���ѡ��ҳ��
!insertmacro MUI_PAGE_COMPONENTS
; ��װĿ¼ѡ��ҳ��
!insertmacro MUI_PAGE_DIRECTORY
; ��װ����ҳ��
!insertmacro MUI_PAGE_INSTFILES
; ��װ���ҳ��
!define MUI_FINISHPAGE_RUN "$INSTDIR\${ExeName}"
!insertmacro MUI_PAGE_FINISH

; ��װж�ع���ҳ��
!insertmacro MUI_UNPAGE_INSTFILES

; ��װ�����������������
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "SimpChinese"

; ��װԤ�ͷ��ļ�
!insertmacro MUI_RESERVEFILE_LANGDLL
!insertmacro MUI_RESERVEFILE_INSTALLOPTIONS
; ------ MUI �ִ����涨����� ------

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "publish\ShowKeyBoard_Setup_${PRODUCT_VERSION}_${DATESTR}.exe"
InstallDir "$PROGRAMFILES\ShowKeyBoard"
InstallDirRegKey HKLM "${PRODUCT_UNINST_KEY}" "UninstallString"
ShowInstDetails show
ShowUnInstDetails show

; ��Ʒ������Ϊ��װж�ص����ݣ�ͬʱ���¼����ѡ���
LangString ToolLang ${LANG_SIMPCHINESE} "���������Ϊ��������"  
LangString ToolLang ${LANG_ENGLISH} "ShowKeyBoard Statistic"

LangString UNINSTALL_SUCC ${LANG_SIMPCHINESE} "�ѳɹ��ش���ļ�����Ƴ���"
LangString UNINSTALL_SUCC ${LANG_ENGLISH} "Uninstall success."

LangString KeyBoardPath ${LANG_SIMPCHINESE} "�������"
LangString KeyBoardPath ${LANG_ENGLISH} "KeyboardMouse"

LangString delConfig ${LANG_ENGLISH} "Delete last config"
LangString delConfig ${LANG_SIMPCHINESE} "ɾ��������"

LangString delConfigDesc ${LANG_ENGLISH} "Config include the parameters of client and Server"
LangString delConfigDesc ${LANG_SIMPCHINESE} "�����ļ������ͻ��˺ͷ���������������Ϣ"

LangString delRecord ${LANG_ENGLISH} "Delete last record"
LangString delRecord ${LANG_SIMPCHINESE} "ɾ����ͳ��"
LangString delRecordDesc ${LANG_ENGLISH} "Records for statistic of mouse, keystroke,keymap,Apps"
LangString delRecordDesc ${LANG_SIMPCHINESE} "ͳ�Ƽ�¼�������̡���ꡢӦ�õ������Ϣ"

LangString unRegMsg ${LANG_ENGLISH} "Whether reserved relative config or records?$\r$\n$\r$\n"
LangString unRegMsg ${LANG_SIMPCHINESE} "�Ƿ������úͼ�¼��$\r$\n$\r$\nȷ����������������YES��������������NO����������鱣����"

LangString mainSect ${LANG_SIMPCHINESE} "����ģ��"
LangString mainSect ${LANG_ENGLISH} "Main"
LangString mainSectDesc ${LANG_SIMPCHINESE} "����ģ�飬���밲װ"
LangString mainSectDesc ${LANG_ENGLISH} "Must be installed"

LangString deskSect ${LANG_SIMPCHINESE} "�����ݷ�ʽ"
LangString deskSect ${LANG_ENGLISH} "Desk shortcut"
LangString deskSectDesc ${LANG_SIMPCHINESE} "���������ݷ�ʽ"
LangString deskSectDesc ${LANG_ENGLISH} "Create desk shortcut"

LangString menuSect ${LANG_SIMPCHINESE} "�˵���ݷ�ʽ"
LangString menuSect ${LANG_ENGLISH} "Menu shortcut"
LangString menuSectDesc ${LANG_SIMPCHINESE} "�����˵���ݷ�ʽ"
LangString menuSectDesc ${LANG_ENGLISH} "Create shortcut in windows menu"

LangString Prepare ${LANG_SIMPCHINESE} "׼��"
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
  SectionIn RO ; �� section ����Ϊֻ�����Խ�ֹ�û�ѡ���ȡ��ѡ��

  SetOutPath "$INSTDIR"
  SetOverwrite on
    File "release\${ExeName}"
    File "release\fixStat�޸�ͳ������.bat"

; �����ļ�������ڲ��ܸ���
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
 *  �����ǰ�װ�����ж�ز���  *
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
    Delete "$INSTDIR\fixStat�޸�ͳ������.bat"
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
  StrCpy $isRunning "��װ�����⵽ ${PRODUCT_NAME} �������С�$\r$\n$\r$\n��� ��ȷ���� ���ԣ�$\r$\n��� ��ȡ���� �˳���ǰ����"
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
; ж�غͰ�װ�����޷�����
Function un.Detect
  StrCpy $isRunning "��װ�����⵽ ${PRODUCT_NAME} �������С�$\r$\n$\r$\n��� ��ȷ���� ���ԣ�$\r$\n��� ��ȡ���� �˳���ǰ����"
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
#-- ���� NSIS �ű��༭�������� Function ���α�������� Section ����֮���д���Ա��ⰲװ�������δ��Ԥ֪�����⡣--#
Function .onInit
  !insertmacro MUI_LANGDLL_DISPLAY
  ReadRegStr $0 HKLM "${PRODUCT_DIR_REGKEY}" ""
  ; ����ҵ���װĿ¼��������ΪĬ�ϰ�װĿ¼
  ${If} $0 != ""
    ${GetParent} $0 $R0
    StrCpy $INSTDIR $R0
  ${EndIf}
  Call Detect
FunctionEnd

Var UNINSTALL_CONFIRM
Function un.onInit
; $LANGUAGE  �������ļ��ж�ȡ����
  StrCpy $UNINSTALL_CONFIRM " ��ȷ��Ҫж�� "
  ReadINIStr $0 "$INSTDIR\showKeyBoard.ini" "common" "defaultLang"
  ${If} $0 == "en-US" ; Ӣ��
    StrCpy $LANGUAGE "1033"
    StrCpy $UNINSTALL_CONFIRM "Are you sure uninstall !"
  ${EndIf}
  ;!insertmacro MUI_UNGETLANGUAGE
  MessageBox MB_ICONQUESTION|MB_YESNO|MB_DEFBUTTON2 "$UNINSTALL_CONFIRM $(^Name) ��" IDYES +2
  Abort
FunctionEnd

Function un.onUninstSuccess
  HideWindow
  MessageBox MB_ICONINFORMATION|MB_OK "$(^Name) $(UNINSTALL_SUCC)"
FunctionEnd
