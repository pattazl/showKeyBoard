// getInputKey.cpp
// 兼容 VS2015 和 VS2022

#include <Windows.h>
#include <string>
#include <unordered_set>
#include <iostream>
#include <fstream>
#include <cstdlib>
#include <queue>
#include <thread>
#include <mutex>
#include <condition_variable>
#include <atomic>

char sendKeySep = '|';  // 链接分隔符,启用了多线程无需合并发送数据
// 全局变量：存储目标窗口标识
HWND g_targetWindowId = 0;
// 全局：存储找到的窗口句柄
HWND g_hWndByPid = NULL;
// 枚举回调
BOOL CALLBACK EnumWindowsProc_GetHwndByPid(HWND hWnd, LPARAM lParam)
{
	DWORD targetPid = (DWORD)lParam;
	DWORD winPid = 0;

	// 获取窗口所属进程 PID
	GetWindowThreadProcessId(hWnd, &winPid);

	if (winPid == targetPid)
	{
		g_hWndByPid = hWnd;  // 匹配成功，保存句柄
		return FALSE;        // 停止枚举
	}
	return TRUE;
}
// 格式化输出调试字符串
void DbgPrint(const wchar_t* format, ...)
{
	const wchar_t prefix[] = L"[KeyInput] "; // 11字符
	const int prefixLen = wcslen(prefix);
	wchar_t buf[512] = { 0 };
	wcscpy_s(buf, _countof(buf), prefix);
	va_list args;
	va_start(args, format);
	vswprintf_s(buf + prefixLen, _countof(buf) - prefixLen, format, args);
	va_end(args);
	OutputDebugStringW(buf);
}
// 通过 PID 获取顶层窗口 HWND（支持隐藏窗口）
HWND GetHwndByPid(DWORD pid)
{
	g_hWndByPid = NULL;
	EnumWindows(EnumWindowsProc_GetHwndByPid, (LPARAM)pid);
	return g_hWndByPid;
}
// -------------------------- 线程安全队列 + 同步变量 --------------------------
// 按键信息结构体
struct KeyInfo {
	std::wstring fullKey;
	int repeatRecord;
};

// 线程安全队列
std::queue<KeyInfo> keyQueue;
// 互斥锁：保护队列访问
std::mutex queueMutex;
// 条件变量：通知消费者线程有新数据
std::condition_variable cv;
// 退出标志：控制线程退出
std::atomic<bool> g_exitFlag(false);

// 回调函数：遍历窗口，匹配标题包含指定字符串的窗口
BOOL CALLBACK EnumWindowsProc(HWND hWnd, LPARAM lParam)
{
	wchar_t szWindowTitle[256] = { 0 };
	GetWindowTextW(hWnd, szWindowTitle, 256);

	// 查找标题中是否包含"ShowKeyBoard.exe"（不区分大小写）
	std::wstring title = szWindowTitle;
	std::wstring target = L"ShowKeyBoard.exe";

	// 转小写
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
	FILE* file = NULL;
	errno_t err = _wfopen_s(&file, pidFilePath.c_str(), L"r");
	if (err == 0 && file != NULL)
	{
		wchar_t buf[32] = { 0 };
		fgetws(buf, _countof(buf), file);
		fclose(file);
		DbgPrint(L"PID in file:%s", buf);
		txtPid = _wtoi(buf);
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
		DbgPrint(L"EnumWindows2FindTarget");
		EnumWindows(EnumWindowsProc, (LPARAM)&tarPid);
	}

	// 5. 调试输出 + 无效PID处理
	if (tarPid == -1)
	{
		DbgPrint(L"CPP: no targetWindow");
		if (initFlag == 1) ExitProcess(0);
		return;
	}
	// 6. 赋值目标窗口标识
	g_targetWindowId =  GetHwndByPid(tarPid);
}
// wstring 全部替换
std::wstring ReplaceAll(std::wstring str, const std::wstring& from, const std::wstring& to)
{
	size_t pos = 0;
	while ((pos = str.find(from, pos)) != std::wstring::npos) {
		str.replace(pos, from.length(), to);
		pos += to.length();
	}
	return str;
}
// -------------------------- 全局配置 --------------------------
volatile int repeatRecord = 0;
int maxKeypressCount = 10;
std::wstring skipKeys = L"{LCtrl}{RCtrl}{LShift}{RShift}{LWin}{RWin}{LAlt}{RAlt}"; // L""; // 

// -------------------------- 辅助函数 --------------------------

/**
* @brief 拼接修饰键（Ctrl/Shift/Alt/Win）
*/
std::wstring AddModifier()
{
	std::wstring mod = L"";
	if (GetAsyncKeyState(VK_LCONTROL) & 0x8000) mod += L"<^";
	if (GetAsyncKeyState(VK_RCONTROL) & 0x8000) mod += L">^";
	if (GetAsyncKeyState(VK_LSHIFT) & 0x8000)   mod += L"<+";
	if (GetAsyncKeyState(VK_RSHIFT) & 0x8000)   mod += L">+";
	if (GetAsyncKeyState(VK_LMENU) & 0x8000)     mod += L"<!";
	if (GetAsyncKeyState(VK_RMENU) & 0x8000)     mod += L">!";
	if (GetAsyncKeyState(VK_LWIN) & 0x8000)     mod += L"<#";
	if (GetAsyncKeyState(VK_RWIN) & 0x8000)     mod += L">#";
	return mod;
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
		return keyNameBuffer;
	}
	return L"Unknown";
}

/**
* @brief 输出（由消费线程执行）
*/
void SendToTargetWindow(const KeyInfo& keyInfo) {
	std::wstring text = keyInfo.fullKey;
	//printf("[KeyPress] %ls | Repeat: %d hwnd:%d\n", keyInfo.fullKey.c_str(), keyInfo.repeatRecord, g_targetWindowId);
	//OutputDebugStringW((L"Pressed: " + keyInfo.fullKey).c_str());
	if (!g_targetWindowId || !IsWindow(g_targetWindowId)) {
		return;
	}
	
	// 构造 COPYDATASTRUCT 用于跨进程 SendMessage
	COPYDATASTRUCT cds{};
	cds.dwData = 0;                  // 自定义数据
	cds.cbData = (DWORD)(text.size() * sizeof(wchar_t)); // 字节长度
	cds.lpData = (LPVOID)text.c_str(); // 数据指针

									   // 发送 WM_COPYDATA（你写的 0x004A 就是 WM_COPYDATA）
	SendMessageTimeout(
		g_targetWindowId,
		WM_COPYDATA,        // 0x004A
		(WPARAM)NULL,
		(LPARAM)&cds,
		SMTO_ABORTIFHUNG,
		4000,               // 超时 4 秒
		NULL
	);
}
/**
* @brief 改造后的PushTxt：将数据推入队列
*/
void PushTxt(const std::wstring& fullKey)
{
	std::lock_guard<std::mutex> lock(queueMutex); // 加锁保护队列
	keyQueue.push({ fullKey, repeatRecord });
	cv.notify_one(); // 通知消费线程有新数据
}

/**
* @brief 消费线程函数：循环读取队列并处理
*/
void ConsumerThreadFunc()
{
	while (!g_exitFlag)
	{
		std::unique_lock<std::mutex> lock(queueMutex);
		// 等待队列有数据 或 收到退出信号
		cv.wait(lock, []() { return !keyQueue.empty() || g_exitFlag; });

		// 退出信号触发且队列为空时，退出线程
		if (g_exitFlag && keyQueue.empty())
		{
			break;
		}

		// 处理队列中的按键信息
		KeyInfo keyInfo = keyQueue.front();
		keyQueue.pop();
		lock.unlock(); // 解锁后再处理，提高并发

		SendToTargetWindow(keyInfo); 
	}
}
// -------------------------- 小键盘按键映射 --------------------------


/**
 * @brief 获取 NumLock 状态
 */
bool IsNumLockOn()
{
	return (GetKeyState(VK_NUMLOCK) & 0x0001) != 0;
}

/**
 * @brief 判断是否是小键盘按键
 *
 * 关键：小键盘按键的 extended 标志为 1
 *       主键盘方向键/Home等的 extended 标志为 0
 */
bool IsNumpadKey(DWORD vkCode, DWORD scancode, DWORD flags)
{
	bool extended = (flags & LLKHF_EXTENDED) != 0;

	// 小键盘运算符（这些键的 extended 可能不同，通过 vkCode 判断）
	if (vkCode == VK_MULTIPLY || vkCode == VK_ADD ||
		vkCode == VK_SUBTRACT || vkCode == VK_DIVIDE ||
		vkCode == VK_DECIMAL || vkCode == VK_NUMLOCK)
	{
		return true;
	}

	// 小键盘回车
	if (vkCode == VK_RETURN && extended)
	{
		return true;
	}

	// 小键盘数字键（NumLock 开启时）
	if (vkCode >= VK_NUMPAD0 && vkCode <= VK_NUMPAD9)
	{
		return true;
	}

	// 【关键】NumLock 关闭时，小键盘按键的 extended 标志为 1
	// 主键盘方向键/Home等的 extended 标志为 0
	// 所以只有 extended == 1 时才可能是小键盘
	if (extended)
	{
		// 检查扫描码是否在小键盘范围内
		switch (scancode)
		{
		case 0x47:  // Home / Numpad7
		case 0x48:  // Up / Numpad8
		case 0x49:  // PgUp / Numpad9
		case 0x4B:  // Left / Numpad4
		case 0x4C:  // Clear / Numpad5
		case 0x4D:  // Right / Numpad6
		case 0x4F:  // End / Numpad1
		case 0x50:  // Down / Numpad2
		case 0x51:  // PgDn / Numpad3
		case 0x52:  // Ins / Numpad0
			return true;
		}
	}

	return false;
}

/**
 * @brief 获取小键盘按键名称
 */
std::wstring GetNumpadKeyName(DWORD vkCode, DWORD scancode, DWORD flags, bool numLockOn)
{
	bool extended = (flags & LLKHF_EXTENDED) != 0;

	// 小键盘运算符
	//switch (vkCode)
	//{
	//case VK_MULTIPLY: return L"NumpadMult";
	//case VK_ADD:      return L"NumpadAdd";
	//case VK_SUBTRACT: return L"NumpadSub";
	//case VK_DIVIDE:   return L"NumpadDiv";
	//case VK_DECIMAL:  return L"NumpadDel";
	//case VK_NUMLOCK:  return L"NumpadNumLock";
	//}

	// 小键盘回车
	if (vkCode == VK_RETURN && extended)
	{
		return L"NumpadEnter";
	}
	if (vkCode == VK_NUMLOCK )
	{
		return L"NumLock";
	}
	// NumLock 开启时，返回数字名称
	if (numLockOn)
	{
		switch (vkCode)
		{
		case VK_NUMPAD0: return L"Numpad0-";
		case VK_NUMPAD1: return L"Numpad1-";
		case VK_NUMPAD2: return L"Numpad2";
		case VK_NUMPAD3: return L"Numpad3";
		case VK_NUMPAD4: return L"Numpad4";
		case VK_NUMPAD5: return L"Numpad5";
		case VK_NUMPAD6: return L"Numpad6";
		case VK_NUMPAD7: return L"Numpad7";
		case VK_NUMPAD8: return L"Numpad8";
		case VK_NUMPAD9: return L"Numpad9";
		case VK_DECIMAL:  return L"NumpadDot";
		}
	}

	// NumLock 关闭时，通过扫描码返回功能名称
	// 注意：只有 extended == 1 时才会进入这个分支
	switch (scancode)
	{
	case 0x52: return L"Insert";   // 小键盘 0
	case 0x4F: return L"End";   // 小键盘 1
	case 0x50: return L"Down";  // 小键盘 2
	case 0x51: return L"PgDn";  // 小键盘 3
	case 0x4B: return L"Left";  // 小键盘 4
	case 0x4C: return L"Clear"; // 小键盘 5
	case 0x4D: return L"Right"; // 小键盘 6
	case 0x47: return L"Home";  // 小键盘 7
	case 0x48: return L"Up";    // 小键盘 8
	case 0x49: return L"PgUp";  // 小键盘 9

	case 0x37:  return L"NumpadMult";    // *
	case 0x4E:  return L"NumpadAdd";     // +
	case 0x4A:  return L"NumpadSub";     // -
	case 0x53:  return L"NumpadDel";     // -
	case 0x35:  if (extended) return L"NumpadDiv"; // /（必须extended=true）

	}

	return L"NumpadUnknown";
}

/**
 * @brief 获取普通按键名称
 */
std::wstring GetNormalKeyName(DWORD vkCode, DWORD scancode, bool extended)
{
	// 特殊按键映射
	switch (vkCode)
	{
	case VK_SPACE:      return L"Space";
	case VK_BACK:       return L"Backspace";
	case VK_TAB:        return L"Tab";
	case VK_ESCAPE:     return L"Esc";
	case VK_DELETE:     return extended?L"Del":L"NumpadDel";
	case VK_INSERT:     return L"NumpadIns";
	case VK_HOME:       return L"NumpadHome";
	case VK_END:        return L"NumpadEnd";
	case VK_PRIOR:      return L"NumpadPgUp";
	case VK_NEXT:       return L"NumpadPgDn";
	case VK_UP:         return L"NumpadUp";
	case VK_DOWN:       return L"NumpadDown";
	case VK_LEFT:       return L"NumpadLeft";
	case VK_RIGHT:      return L"NumpadRight";
	case VK_CONTROL:
		return extended ? L"RCtrl" : L"LCtrl";
	case VK_SHIFT:
		// 通过扫描码区分左右 Shift
		if (scancode == 0x2A) return L"LShift";
		if (scancode == 0x36) return L"RShift";
		return L"Shift";
	case VK_MENU:  // Alt
		return extended ? L"RAlt" : L"LAlt";
	case VK_LWIN:       return L"LWin";
	case VK_RWIN:       return L"RWin";
	case VK_APPS:       return L"AppsKey";
	case VK_CAPITAL:    return L"CapsLock";
	case VK_SCROLL:     return L"ScrollLock";
	case VK_SNAPSHOT:   return L"PrintScreen";
	case VK_PAUSE:      return L"Pause";

	case VK_LCONTROL: return L"LCtrl";
	case VK_RCONTROL: return L"RCtrl";
	case VK_LSHIFT:  return L"LShift";
	case VK_RSHIFT:  return L"RShift";
	case VK_LMENU:   return L"LAlt";  
	case VK_RMENU:   return L"RAlt";  
	}

	// 字母键 A-Z
	if (vkCode >= 'A' && vkCode <= 'Z')
	{
		// 大写转小写：+ 32
		wchar_t ch = (wchar_t)(vkCode +32);
		return std::wstring(1, ch);
	}

	// 数字键 0-9（主键盘区）
	if (vkCode >= '0' && vkCode <= '9')
	{
		wchar_t ch = (wchar_t)vkCode;
		return std::wstring(1, ch);
	}

	// F1-F24
	if (vkCode >= VK_F1 && vkCode <= VK_F24)
	{
		wchar_t buf[16];
		swprintf_s(buf, L"F%d", vkCode - VK_F1 + 1);
		return buf;
	}
	// 通过扫描码获取名称（兜底）
	return GetKeyNameFromSC(scancode, extended);

}
/**
 * @brief 判断扫描码是否属于小键盘按键
 */


std::wstring GetKeyNameVCode(KBDLLHOOKSTRUCT* pKeyboardData) {
	DWORD vkCode = pKeyboardData->vkCode;
	DWORD scanCode = pKeyboardData->scanCode;
	DWORD flags = pKeyboardData->flags;
	bool extended = !!(pKeyboardData->flags & LLKHF_EXTENDED);
	if (IsNumpadKey(vkCode, scanCode, flags))
	{
		bool numLockOn = IsNumLockOn();
		return GetNumpadKeyName(vkCode, scanCode, flags, numLockOn);
	}
	// 普通按键
	return GetNormalKeyName(vkCode, scanCode, extended);
}
// -------------------------- 核心钩子回调函数 --------------------------
LRESULT CALLBACK LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam)
{
	if (nCode < 0)
	{
		return CallNextHookEx(NULL, nCode, wParam, lParam);
	}

	KBDLLHOOKSTRUCT* pKeyboardData = reinterpret_cast<KBDLLHOOKSTRUCT*>(lParam);
	DWORD vk = pKeyboardData->vkCode;

	bool isKeyDown = (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN);
	bool isKeyUp = (wParam == WM_KEYUP || wParam == WM_SYSKEYUP);

	// 1. 按键抬起：重置计数
	if (isKeyUp)
	{
		repeatRecord = 0;
	}

	// 2. 处理物理按键按下
	if (repeatRecord < maxKeypressCount)
	{
		bool isInjected = (pKeyboardData->flags & LLKHF_INJECTED) != 0;
		if (!isInjected && isKeyDown)
		{
			bool extended = (pKeyboardData->flags & LLKHF_EXTENDED) != 0;

			std::wstring Name = GetKeyNameVCode(pKeyboardData);
			// std::wstring Name = GetKeyNameFromSC(pKeyboardData->scanCode, extended);
			std::wstring Name2 = L"{" + Name + L"}";
			// wprintf(Name2.c_str());
			// 跳过指定按键
			if (skipKeys.find(Name2) != std::wstring::npos )
			{
				return CallNextHookEx(NULL, nCode, wParam, lParam);
			}

			repeatRecord++;
			std::wstring fullKey = AddModifier() + Name;
			//wprintf(fullKey.c_str());
			PushTxt(fullKey); // 仅推入队列，不直接输出
		}
	}

	return CallNextHookEx(NULL, nCode, wParam, lParam);
}
// 窗口过程（必须！处理关闭、退出消息）
LRESULT CALLBACK HiddenWndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam)
{
	switch (msg)
	{
		// 接收关闭消息（AHK 可发送 WM_CLOSE）
	case WM_CLOSE:
		DestroyWindow(hWnd);
		break;

		// 窗口销毁 → 发送退出消息，结束消息循环
	case WM_DESTROY:
		PostQuitMessage(0);
		break;

	default:
		return DefWindowProc(hWnd, msg, wParam, lParam);
	}
	return 0;
}

// ==============================================
// 创建隐藏窗口（在 main 开头调用）
// ==============================================
bool CreateHiddenWindow()
{
	HINSTANCE hInstance = GetModuleHandle(NULL);

	// 1. 注册窗口类
	WNDCLASSEX wc = { 0 };
	wc.cbSize = sizeof(WNDCLASSEX);
	wc.lpfnWndProc = HiddenWndProc;
	wc.hInstance = hInstance;
	wc.lpszClassName = L"ShowKeyBoardGetKeyInputCls";
	if (!RegisterClassEx(&wc))
		return false;

	// 2. 创建隐藏窗口（无标题、隐藏、不激活）
	HWND hHiddenWnd = CreateWindowEx(
		0,
		wc.lpszClassName,
		L"HiddenWindow",
		0,            // 无样式
		0, 0, 0, 0,   // 宽高0
		NULL, NULL, hInstance, NULL
	);

	return hHiddenWnd != NULL;
}
// 唯一标识（随便写一个唯一字符串即可，不要和别人重复）
#define UNIQUE_MUTEX_NAME L"GetKeyInput_8765ABCD"

// ==============================================
// 函数：确保程序只运行一个实例（单例守护）
// 返回值：true = 可以启动 | false = 已经在运行，直接退出
// ==============================================
bool CheckSingleInstance()
{
	// 创建互斥体
	HANDLE hMutex = CreateMutexW(NULL, TRUE, UNIQUE_MUTEX_NAME);

	if (hMutex != NULL)
	{
		// 已经存在实例
		if (GetLastError() == ERROR_ALREADY_EXISTS)
		{
			CloseHandle(hMutex); // 安全关闭
			return false;
		}
		return true;
	}
	return true;
}

// -------------------------- 主函数 --------------------------
// 正确的 wWinMain 签名（与 SDK 一致）
int WINAPI wWinMain(
	_In_ HINSTANCE hInstance,
	_In_opt_ HINSTANCE hPrevInstance,
	_In_ LPWSTR lpCmdLine,
	_In_ int nCmdShow
)
{
	// 设置控制台为 UTF-8 编码
	//SetConsoleOutputCP(CP_UTF8);
	//SetConsoleCP(CP_UTF8);
	// 需要防止重复打开
	if (!CheckSingleInstance())
	{
		DbgPrint(L"Exist Instance!!");
		return -1;
	}
	// ====================== 获取参数 ======================
	LPWSTR cmdLine = GetCommandLineW();
	int argc;
	LPWSTR* argv = CommandLineToArgvW(cmdLine, &argc);
	if (argc > 2)
	{
		// 第1个参数：转成 int 数字
		maxKeypressCount = _wtoi(argv[1]);
		// 第2个参数：字符串
		skipKeys = argv[2];
		DbgPrint( L"maxKeypressCount:%d,keys:%s", maxKeypressCount,skipKeys.c_str());
	}
	// 需要增加隐藏窗口用于获取窗口句柄发消息关闭
	CreateHiddenWindow();


	// 获取目标进程 PID
	getMainPid(1);
	// 启动消费线程
	std::thread consumerThread(ConsumerThreadFunc);
	// 安装键盘钩子
	HHOOK hKeyboardHook = SetWindowsHookExW(WH_KEYBOARD_LL,
		LowLevelKeyboardProc,
		GetModuleHandleW(NULL),
		0);

	if (hKeyboardHook == NULL)
	{
		DbgPrint(L"安装键盘钩子失败！错误码：%d", GetLastError());
		// 触发退出信号
		g_exitFlag = true;
		cv.notify_one();
		if (consumerThread.joinable())
		{
			consumerThread.join();
		}
		return 1;
	}
	DbgPrint(L"键盘钩子安装成功！按 Ctrl+C 退出程序...");
	// DbgPrint(L"测试：按下空格键/Shift+空格/Ctrl+空格，可看到输出\n");

	// 消息循环
	MSG msg;
	while (GetMessage(&msg, NULL, 0, 0))
	{
		TranslateMessage(&msg);
		DispatchMessage(&msg);
	}
	// 程序退出前的清理
	g_exitFlag = true; // 设置退出标志
	cv.notify_one();   // 唤醒消费线程
	if (consumerThread.joinable())
	{
		consumerThread.join(); // 等待线程结束
	}

	UnhookWindowsHookEx(hKeyboardHook);
	return 0;
}