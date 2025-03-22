用于发布的脚本和目录，使用 nsis打包工具执行 showKeyBoard.nsi 即可
release目录用于输出安装包

nsis需要安装 LogicLib.nsh 和 nsProcess 库

如果需要编译打包,请执行如下步骤
1. 切换项目根目录下
2. 修改 package.json中的版本号
3. 复制本目录的 buildMain.config.js.dist 为 buildMain.config.js
4. 修改 buildMain.config.js 中的编译脚本目录
5. 根目录下 执行 npm run build ,注意:第一次需执行 npm install
6. 将自动调用本目录的buildMain.js 文件