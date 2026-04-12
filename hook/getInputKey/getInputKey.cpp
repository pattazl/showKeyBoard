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

char sendKeySep = '|';  // 链接分隔符
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

			printf("%s", pidStr);
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
	if (tarPid == -1)
	{
		OutputDebugStringW(L"CPP: no targetWindow");
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
std::wstring skipKeys = L""; // L"{LCtrl}{RCtrl}{LShift}{RShift}{LWin}{RWin}{LAlt}{RAlt}";

							 // 全局变量：记录当前按住的修饰键（左右分开）
bool g_bLControl = false;
bool g_bRControl = false;
bool g_bLShift = false;
bool g_bRShift = false;
bool g_bLAlt = false;
bool g_bRAlt = false;
bool g_bLWin = false;
bool g_bRWin = false;
// -------------------------- 辅助函数 --------------------------

/**
* @brief 拼接修饰键（Ctrl/Shift/Alt/Win）
*/
std::wstring AddModifier()
{
	std::wstring mod = L"";
	if (g_bLControl) mod += L"<^";
	if (g_bRControl) mod += L">^";
	if (g_bLShift)   mod += L"<+";
	if (g_bRShift)   mod += L">+";
	if (g_bLAlt)     mod += L"<!";
	if (g_bRAlt)     mod += L">!";
	if (g_bLWin)     mod += L"<#";
	if (g_bRWin)     mod += L">#";

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
std::wstring GetKeyNameVCode(KBDLLHOOKSTRUCT* pKeyboardData) {
	DWORD vkCode = pKeyboardData->vkCode;
	bool extended = !!(pKeyboardData->flags & LLKHF_EXTENDED);
	std::wstring keyName;
	// ====================== 正确识别：控制键 + 小键盘（永不错乱）======================
	switch (vkCode)
	{
		// 控制键
	case VK_LCONTROL: keyName = L"LControl"; break;
	case VK_RCONTROL: keyName = L"RControl"; break;
	case VK_LSHIFT:  keyName = L"LShift";  break;
	case VK_RSHIFT:  keyName = L"RShift";  break;
	case VK_LMENU:   keyName = L"LAlt";    break;
	case VK_RMENU:   keyName = L"RAlt";    break;
	case VK_LWIN:    keyName = L"LWin";    break;
	case VK_RWIN:    keyName = L"RWin";    break;
	case VK_APPS:    keyName = L"App";     break;

		// 功能键
	case VK_ESCAPE:    keyName = L"Esc";     break;
	case VK_TAB:       keyName = L"Tab";     break;
	case VK_CAPITAL:   keyName = L"CapsLock"; break;
	case VK_SPACE:     keyName = L"Space";   break;
	case VK_BACK:      keyName = L"Backspace"; break;
	case VK_RETURN:    keyName = L"Enter";   break;
	case VK_INSERT:    keyName = L"Insert";  break;
	case VK_DELETE:    keyName = L"Delete";  break;
	case VK_HOME:      keyName = L"Home";    break;
	case VK_END:       keyName = L"End";     break;
	case VK_PRIOR:     keyName = L"PageUp";  break;
	case VK_NEXT:      keyName = L"PageDown"; break;
	case VK_UP:        keyName = L"Up";      break;
	case VK_LEFT:      keyName = L"Left";    break;
	case VK_RIGHT:     keyName = L"Right";   break;
	case VK_DOWN:      keyName = L"Down";    break;

		// 小键盘（完全正确区分）
	case VK_NUMPAD0: keyName = L"Num0"; break;
	case VK_NUMPAD1: keyName = L"Num1"; break;
	case VK_NUMPAD2: keyName = L"Num2"; break;
	case VK_NUMPAD3: keyName = L"Num3"; break;
	case VK_NUMPAD4: keyName = L"Num4"; break;
	case VK_NUMPAD5: keyName = L"Num5"; break;
	case VK_NUMPAD6: keyName = L"Num6"; break;
	case VK_NUMPAD7: keyName = L"Num7"; break;
	case VK_NUMPAD8: keyName = L"Num8"; break;
	case VK_NUMPAD9: keyName = L"Num9"; break;
	case VK_MULTIPLY: keyName = L"Num*"; break;
	case VK_ADD:      keyName = L"Num+"; break;
	case VK_SUBTRACT: keyName = L"Num-"; break;
	case VK_DECIMAL:  keyName = L"Num."; break;
	case VK_DIVIDE:   keyName = L"Num/"; break;

	default:
		// 普通按键 字母/数字/符号 走 GetKeyNameFromSC
		keyName = GetKeyNameFromSC(pKeyboardData->scanCode, extended);
		break;
	}
	return keyName;
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

	// ====================== 更新全局修饰键状态 ======================
	if (vk == VK_LCONTROL) g_bLControl = isKeyDown;
	if (vk == VK_RCONTROL) g_bRControl = isKeyDown;
	if (vk == VK_LSHIFT)   g_bLShift = isKeyDown;
	if (vk == VK_RSHIFT)   g_bRShift = isKeyDown;
	if (vk == VK_LMENU)    g_bLAlt = isKeyDown;
	if (vk == VK_RMENU)    g_bRAlt = isKeyDown;
	if (vk == VK_LWIN)     g_bLWin = isKeyDown;
	if (vk == VK_RWIN)     g_bRWin = isKeyDown;
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
			wprintf(Name2.c_str());
			// 跳过指定按键
			if (skipKeys.find(Name2) != std::wstring::npos )
			{
				return CallNextHookEx(NULL, nCode, wParam, lParam);
			}

			repeatRecord++;
			std::wstring fullKey = AddModifier() + Name;

			PushTxt(fullKey); // 仅推入队列，不直接输出
		}
	}

	return CallNextHookEx(NULL, nCode, wParam, lParam);
}

// -------------------------- 主函数 --------------------------
int main(int argc, char* argv[])
{
	// 设置控制台为 UTF-8 编码
	SetConsoleOutputCP(CP_UTF8);
	SetConsoleCP(CP_UTF8);
	// ====================== 获取参数 ======================
	if (argc >2)
	{
		// 第1个参数：转成 int 数字
		maxKeypressCount = std::atoi(argv[1]);
		// 第2个参数：字符串
		std::string str = argv[2];
		skipKeys = std::wstring(str.begin(), str.end());
		// wprintf( L"count:%d,keys:%s", maxKeypressCount,skipKeys.c_str());
	}
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
		printf("安装键盘钩子失败！错误码：%d\n", GetLastError());
		// 触发退出信号
		g_exitFlag = true;
		cv.notify_one();
		if (consumerThread.joinable())
		{
			consumerThread.join();
		}
		return 1;
	}
	printf("键盘钩子安装成功！按 Ctrl+C 退出程序...\n");
	printf("测试：按下空格键/Shift+空格/Ctrl+空格，可看到输出\n");

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