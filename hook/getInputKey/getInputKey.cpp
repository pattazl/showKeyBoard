#include <Windows.h>
#include <string>
#include <unordered_set>
#include <iostream>
#include <locale>
#include <codecvt>

// -------------------------- 全局配置 --------------------------
volatile int repeatRecord = 0;
const int maxKeypressCount = 10;
// 修复：移除{Space}，避免空格被跳过；保留其他示例跳过键
const std::unordered_set<std::wstring> skipKeys = {
    // L"{Esc}", L"{Tab}", L"{Backspace}", L"{Delete}", L"{Enter}"
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
    if (GetKeyState(VK_LWIN) & 0x8000 || GetKeyState(VK_RWIN) & 0x8000) modifier += L"Win+";
    return modifier;
}

/**
 * @brief 修复：强化按键名映射，确保空格正确识别
 */
std::wstring GetKeyNameFromSC(DWORD scancode, bool extended)
{
    WCHAR keyNameBuffer[256] = { 0 };
    DWORD scancodeExtended = extended ? (scancode | 0xE000) : scancode;

    if (GetKeyNameTextW(scancodeExtended << 16, keyNameBuffer, _countof(keyNameBuffer)))
    {
        std::wstring keyName = keyNameBuffer;
        // 空格按键特殊处理（覆盖所有可能的名称）
        if (keyName == L"空格键" || scancode == 0x39) return L"Space";
        // 其他按键统一
        if (keyName == L"左Ctrl键") return L"Ctrl";
        if (keyName == L"右Ctrl键") return L"Ctrl";
        if (keyName == L"左Shift键") return L"Shift";
        if (keyName == L"右Shift键") return L"Shift";
        if (keyName == L"左Alt键") return L"Alt";
        if (keyName == L"右Alt键") return L"Alt";
        if (keyName == L"左Win键") return L"Win";
        if (keyName == L"右Win键") return L"Win";
        if (keyName == L"回车键") return L"Enter";
        return keyName;
    }
    // 兜底：通过扫描码直接识别空格（0x39是空格的扫描码）
    // if (scancode == 0x39) return L"Space";
    return L"Unknown";
}

/**
 * @brief 输出按键信息
 */
void PushTxt(const std::wstring& fullKey)
{
    std::wcout << L"[KeyPress] " << fullKey << L" | Repeat: " << repeatRecord << std::endl;
    // 调试输出（可选）
    // OutputDebugStringW((L"Pressed: " + fullKey).c_str());
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
            DWORD sc = (extended << 8) | pKeyboardData->scanCode;

            std::wstring Name = GetKeyNameFromSC(pKeyboardData->scanCode, extended);
            std::wstring Name2 = L"{" + Name + L"}";

            // 跳过指定按键（已移除{Space}）
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
    // 修复：优化控制台编码，确保Space等字符正常显示
    //SetConsoleOutputCP(CP_UTF8);
    //SetConsoleCP(CP_UTF8);
    std::wcout.imbue(std::locale(std::locale::empty(), new std::codecvt_utf8<wchar_t>));

    HHOOK hKeyboardHook = SetWindowsHookEx(WH_KEYBOARD_LL,
        LowLevelKeyboardProc,
        GetModuleHandle(NULL),
        0);
    if (hKeyboardHook == NULL)
    {
        std::cerr << "安装键盘钩子失败！错误码：" << GetLastError() << std::endl;
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