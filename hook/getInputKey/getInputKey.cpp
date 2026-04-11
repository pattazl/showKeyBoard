// getInputKey.cpp
// 兼容 VS2015 和 VS2022

#include <Windows.h>
#include <string>
#include <unordered_set>
#include <iostream>
#include <locale>
#include <fstream>
#include <algorithm>
#include <codecvt>
#include <cstdlib>

// 全局变量：存储目标窗口标识
long g_targetWindowId = 0;

// 回调函数：遍历窗口，匹配标题包含指定字符串的窗口
BOOL CALLBACK EnumWindowsProc(HWND hWnd, LPARAM lParam)
{
    wchar_t szWindowTitle[256] = { 0 };
    GetWindowTextW(hWnd, szWindowTitle, 256);

    // 查找标题中是否包含"ShowKeyBoard.exe"（不区分大小写）
    std::wstring title = szWindowTitle;
    std::wstring target = L"ShowKeyBoard.exe";

    // VS2015 兼容的转小写方式
    for (auto& c : title) c = towlower(c);
    for (auto& c : target) c = towlower(c);

    if (title.find(target) != std::wstring::npos)
    {
        DWORD* pPid = (DWORD*)lParam;
        GetWindowThreadProcessId(hWnd, pPid);
        return FALSE;
    }
    return TRUE;
}

// 核心函数：获取目标进程PID（按窗口标题匹配）
void getMainPid(int initFlag)
{
    // 1. 拼接PID文件路径（程序目录下kb.pid）
    WCHAR szExePath[MAX_PATH] = { 0 };
    GetModuleFileNameW(NULL, szExePath, MAX_PATH);
    std::wstring pidFilePath = szExePath;
    size_t lastSlash = pidFilePath.find_last_of(L"\\/");
    if (lastSlash != std::wstring::npos) {
        pidFilePath = pidFilePath.substr(0, lastSlash) + L"\\kb.pid";
    }
    else {
        pidFilePath = L"kb.pid";
    }

    // 2. 读取PID文件
    int txtPid = -1;
    
    // VS2015 兼容的文件打开方式
    FILE* file = NULL;
    _wfopen_s(&file, pidFilePath.c_str(), L"r");
    if (file != NULL)
    {
        char pidStr[32] = { 0 };
        if (fgets(pidStr, sizeof(pidStr), file) != NULL)
        {
            // 去除换行符
            char* newline = strchr(pidStr, '\n');
            if (newline) *newline = '\0';
            newline = strchr(pidStr, '\r');
            if (newline) *newline = '\0';
            
            std::cout << pidStr;
            txtPid = atoi(pidStr);
        }
        fclose(file);
    }

    // 3. 验证PID有效性（按PID查窗口）
    DWORD tarPid = -1;
    HWND hWnd = NULL;
    while ((hWnd = FindWindowExW(NULL, hWnd, NULL, NULL)) != NULL)
    {
        DWORD dwPid = 0;
        GetWindowThreadProcessId(hWnd, &dwPid);
        if (dwPid == (DWORD)txtPid && txtPid != -1)
        {
            tarPid = txtPid;
            break;
        }
    }

    // 4. PID无效时：按窗口标题包含"ShowKeyBoard.exe"查找
    if (tarPid == -1)
    {
        EnumWindows(EnumWindowsProc, (LPARAM)&tarPid);
    }

    // 5. 调试输出 + 无效PID处理
    std::wstring debugMsg = L"CPP: tarPid=" + std::to_wstring(tarPid);
    OutputDebugStringW(debugMsg.c_str());
    
    if (tarPid == -1)
    {
        OutputDebugStringW(L"CPP: no targetWindow");
        if (initFlag == 1) ExitProcess(0);
        return;
    }

    // 6. 赋值目标窗口标识
    g_targetWindowId = tarPid;
}

// -------------------------- 全局配置 --------------------------
volatile int repeatRecord = 0;
const int maxKeypressCount = 10;
const std::unordered_set<std::wstring> skipKeys = {
    // 跳过键列表（为空）
};

// -------------------------- 辅助函数 --------------------------

/**
 * @brief 拼接修饰键（Ctrl/Shift/Alt/Win）
 */
std::wstring AddModifier()
{
    std::wstring modifier;
    if (GetKeyState(VK_CONTROL) & 0x8000) modifier += L"Ctrl+";
    if (GetKeyState(VK_SHIFT) & 0x8000) modifier += L"Shift+";
    if (GetKeyState(VK_MENU) & 0x8000) modifier += L"Alt+";
    if ((GetKeyState(VK_LWIN) & 0x8000) || (GetKeyState(VK_RWIN) & 0x8000)) modifier += L"Win+";
    return modifier;
}

/**
 * @brief 获取按键名称
 */
std::wstring GetKeyNameFromSC(DWORD scancode, bool extended)
{
    WCHAR keyNameBuffer[256] = { 0 };
    DWORD scancodeExtended = extended ? (scancode | 0xE000) : scancode;

    if (GetKeyNameTextW(scancodeExtended << 16, keyNameBuffer, _countof(keyNameBuffer)))
    {
        std::wstring keyName = keyNameBuffer;
        
        // 按键名称映射
        if (keyName == L"空格键" || scancode == 0x39) return L"Space";
        if (keyName == L"左Ctrl键") return L"Ctrl";
        if (keyName == L"右Ctrl键") return L"Ctrl";
        if (keyName == L"左Shift键") return L"Shift";
        if (keyName == L"右Shift键") return L"Shift";
        if (keyName == L"左Alt键") return L"Alt";
        if (keyName == L"右Alt键") return L"Alt";
        if (keyName == L"左Win键") return L"Win";
        if (keyName == L"右Win键") return L"Win";
        if (keyName == L"回车键") return L"Enter";
        if (keyName == L"Backspace") return L"Backspace";
        if (keyName == L"Tab") return L"Tab";
        
        return keyName;
    }
    
    // 通过扫描码识别空格
    if (scancode == 0x39) return L"Space";
    return L"Unknown";
}

/**
 * @brief 输出按键信息
 */
void PushTxt(const std::wstring& fullKey)
{
    std::wcout << L"[KeyPress] " << fullKey << L" | Repeat: " << repeatRecord << std::endl;
    OutputDebugStringW((L"Pressed: " + fullKey).c_str());
}

// -------------------------- 核心钩子回调函数 --------------------------
LRESULT CALLBACK LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam)
{
    if (nCode < 0)
    {
        return CallNextHookEx(NULL, nCode, wParam, lParam);
    }

    KBDLLHOOKSTRUCT* pKeyboardData = reinterpret_cast<KBDLLHOOKSTRUCT*>(lParam);

    // 1. 按键抬起：重置计数
    if (wParam == WM_KEYUP || wParam == WM_SYSKEYUP)
    {
        repeatRecord = 0;
    }

    // 2. 处理物理按键按下
    if (repeatRecord < maxKeypressCount)
    {
        bool isInjected = (pKeyboardData->flags & LLKHF_INJECTED) != 0;
        if (!isInjected && (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN))
        {
            bool extended = (pKeyboardData->flags & LLKHF_EXTENDED) != 0;
            DWORD sc = pKeyboardData->scanCode;

            std::wstring Name = GetKeyNameFromSC(pKeyboardData->scanCode, extended);
            std::wstring Name2 = L"{" + Name + L"}";

            // 跳过指定按键
            if (skipKeys.find(Name2) != skipKeys.end())
            {
                return CallNextHookEx(NULL, nCode, wParam, lParam);
            }

            repeatRecord++;
            std::wstring fullKey = AddModifier() + Name;

            PushTxt(fullKey);
        }
    }

    return CallNextHookEx(NULL, nCode, wParam, lParam);
}

// -------------------------- 主函数 --------------------------
int main()
{
    // 设置控制台编码为 UTF-8（VS2015 兼容方式）
    SetConsoleOutputCP(CP_UTF8);
    SetConsoleCP(CP_UTF8);
    
    // 设置 wcout 的 locale
    std::locale::global(std::locale(""));
    std::wcout.imbue(std::locale());

    // 获取目标进程 PID
    getMainPid(1);

    // 安装键盘钩子
    HHOOK hKeyboardHook = SetWindowsHookExW(WH_KEYBOARD_LL,
        LowLevelKeyboardProc,
        GetModuleHandleW(NULL),
        0);
    
    if (hKeyboardHook == NULL)
    {
        std::cerr << "安装键盘钩子失败！错误码：" << GetLastError() << std::endl;
        system("pause");
        return 1;
    }

    std::cout << "键盘钩子安装成功！按 Ctrl+C 退出程序..." << std::endl;
    std::cout << "测试：按下空格键/Shift+空格/Ctrl+空格，可看到输出" << std::endl;

    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0))
    {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
    
    UnhookWindowsHookEx(hKeyboardHook);
    return 0;
}