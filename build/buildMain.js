// 统一修改showkeyboard.ahk 和 .nsi文件 , 此文件从主目录的运行 npm run build 调用
const { Ahk2Exe, NSIS } = require('./buildMain.config.js');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const iconv = require('iconv-lite');
// 主程序
(() => {
    // 定义文件路径
    const package = path.resolve(__dirname, '../', 'package.json');
    const nsiFileNode = path.resolve(__dirname, 'showKeyBoard(node).nsi');;
    const nsiFile = path.resolve(__dirname, 'showKeyBoard.nsi');
    const ahkFile = path.resolve(__dirname, '../', 'showKeyBoard.ahk');
    console.log('统一根据package.json的版本修改 showkeyboard.ahk 和 .nsi文件')
    // 获取版本 
    let version = getVersion(package)
    // 读取文件和替换版本
    // !define PRODUCT_VERSION "v1.38"
    let res = false
    res = replaceVer([nsiFileNode, nsiFile], true, version, [/!define PRODUCT_VERSION "v(\d+\.\d+)"/]) 
    if(!res)return
// ;@Ahk2Exe-SetProductVersion 1.38.0.0
// ;@Ahk2Exe-SetFileVersion 1.38.0.0
// global APPName:="ShowKeyBoard", ver:="1.38" 
    res = replaceVer([ahkFile], false, version,[
        /;@Ahk2Exe-SetProductVersion (\d+\.\d+)/,
        /;@Ahk2Exe-SetFileVersion (\d+\.\d+)/,
        /global\s+APPName\s*:=\s*"ShowKeyBoard",\s*ver\s*:=\s*"(\d+\.\d+)"/,
    ])
    if(!res)return
    // 执行编译
    if (!fs.existsSync(Ahk2Exe)) {
        console.log('!!!AHK编译文件不存在,无法编译执行文件:',Ahk2Exe);
        return
    }
    // 编译 AHK
    // 构建编译命令
    res = myExec(`"${Ahk2Exe}" /in "${ahkFile}" `,'AHK',1) // 可以输出编译明细
    if(!res)return

    // 编译 NSIS
    if (!fs.existsSync(NSIS)) {
        console.log('!!!NSIS编译文件不存在,无法编译安装包:',NSIS);
        return
    }
    res = myExec(`"${NSIS}" "${nsiFileNode}" `,'NSIS Node')
    if(!res)return
    res = myExec(`"${NSIS}" "${nsiFile}" `,'NSIS')
    if(!res)return
    // 编译NSIS

})();

// 编译调用
function myExec(command,info,detail){
    try {
        // 同步执行命令
        const output = execSync(command);
        console.log(`编译 ${info} 成功`);
        if(detail){
            console.log(`输出信息: ${output?.toString()}`);
        }
    } catch (error) {
        console.error(`编译 ${info} 出错: ${error.message}`);
        console.error(`命令执行过程中出现错误: ${error.stderr?.toString()}`);
        return false
    }
    return true
}

// 替换文件中的 {{mainVer}} 为package中的版本号
function getVersion(packFile) {
    // 读取主目录的 package.json 的版本为主版本
    const packageJsonContent = fs.readFileSync(packFile, 'utf-8');
    // 解析 JSON
    const packageJson = JSON.parse(packageJsonContent);
    console.log('Version:', packageJson.version)
    return packageJson.version
}
function replaceVer(filePath, isGbk, ver,RegExps) {
    let res = true
    try {
        // 以二进制模式同步读取文件
        res = filePath.every(file => {
            if (!fs.existsSync(file)) {
                console.log('文件不存在:', file);
                return false
            }
            let content = ''
            // 将 gbk/ GB2312 编码的二进制数据转换为 UTF-8 字符串
            if (isGbk) {
                const buffer = fs.readFileSync(file);
                content = iconv.decode(buffer, 'gbk');
            } else {
                content = fs.readFileSync(file, 'utf-8');
            }
            let newContent = content
            RegExps.forEach(reg => {
                // 只替换子匹配
                newContent = newContent.replace(reg, (match, a) => {
                    let result = match;
                    result = result.replace(a, ver);
                    return result;
                })
            });
            // 写入原文件
            if (isGbk) {
                const encodedData = iconv.encode(newContent, 'gbk');
                // 将二进制数据写入文件
                fs.writeFileSync(file, encodedData);
            } else {
                fs.writeFileSync(file, newContent);
            }
            return true
        });
    } catch (err) {
        console.error('读取或写入文件时版本时出错:', err);
    }
    return res;
}


