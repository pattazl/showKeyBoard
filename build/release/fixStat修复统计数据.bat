@echo off
cd ./httpdist/dist/
echo Can be repeatedly,Running... 可重复运行,运行清理中...
node.exe server.js cleanErrAppStat
pause