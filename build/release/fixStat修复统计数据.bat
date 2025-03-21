@echo off
cd ./httpdist/dist/
echo Running...运行清理中...
node.exe server.js cleanErrAppStat
pause