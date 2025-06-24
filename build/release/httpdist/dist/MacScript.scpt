-- 获取当前脚本的路径
set currentScriptPath to (path to me as text)
-- 提取脚本所在的文件夹路径
set scriptFolder to POSIX path of (currentScriptPath & "::")

tell application "Terminal"
-- 创建一个新的终端窗口
    -- set newWindow to do script ""
    -- 在新窗口中执行 node 命令
    -- do script "./node" -- in newWindow
    do script "cd " & quoted form of scriptFolder & "; ./node server.js"
    -- 激活终端应用程序，使其显示在前台
    -- activate
end tell