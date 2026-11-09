// 根据 rust/client/Cargo.toml 的主版本号更新 .nsi，编译 release 产物并打包安装包与便携版
// 此文件从 rust 目录运行 npm run build 调用
// npm run pack 传入 --pack：跳过 cargo 编译与产物替换，直接用 build/release 现有文件打包
const { NSIS, ZIP7 } = require('./buildMain.config.js');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const iconv = require('iconv-lite');
// 主程序
(() => {
    // 是否跳过 cargo 编译（npm run pack）
    const skipCargo = process.argv.includes('--pack')
    // 定义文件路径
    const clientDir = path.resolve(__dirname, '../client');
    const serverDir = path.resolve(__dirname, '../server');
    const cargoToml = path.join(clientDir, 'Cargo.toml');
    const nsiFile = path.resolve(__dirname, 'showKeyBoard_rust.nsi');
    const releaseDir = path.resolve(__dirname, 'release');
    const distDir = path.join(releaseDir, 'httpdist', 'dist');
    const publishDir = path.resolve(__dirname, 'publish');

    console.log(skipCargo ? '打包模式：跳过 cargo 编译' : '编译模式：编译 release 产物后打包')
    console.log('根据 client/Cargo.toml 的版本修改 .nsi 文件')
    // 获取版本（取前两位，例如 2.0.0 -> 2.0）
    let version = getVersion(cargoToml)
    if (!version) return

    // 1. 检查 publish 目录，不存在则创建
    ensureDir(publishDir)

    // 2. 编译 client/server 的 release 产物，并替换到 release 目录
    if (!skipCargo) {
        let res = myExec('cargo build --release', 'client release', false, clientDir)
        if (!res) return
        res = myExec('cargo build --release', 'server release', false, serverDir)
        if (!res) return

        res = copyFile(
            path.join(clientDir, 'target', 'release', 'showkeyboard_client.exe'),
            path.join(releaseDir, 'showkeyboard_client.exe')
        )
        if (!res) return
        res = copyFile(
            path.join(serverDir, 'target', 'release', 'showkeyboard_server.exe'),
            path.join(distDir, 'showkeyboard_server.exe')
        )
        if (!res) return
    }

    // 读取文件和替换版本
    // !define PRODUCT_VERSION "v2.0"
    let res = replaceVer([nsiFile], true, version, [/!define PRODUCT_VERSION "v(\d+\.\d+)"/])
    if (!res) return

    // 编译 NSIS（nsi 内的 OutFile/File 使用相对路径，cwd 必须为 build 目录）
    if (fs.existsSync(NSIS)) {
        myExec(`"${NSIS}" "${nsiFile}" `, 'NSIS', false, __dirname)
    } else {
        console.log('!!!NSIS编译文件不存在,跳过安装包:', NSIS)
    }

    // 3. 用 7zip 最高级别压缩 release 目录，输出便携版到 publish 目录
    if (!fs.existsSync(ZIP7)) {
        console.log('!!!7zip编译文件不存在,跳过便携版打包:', ZIP7)
        return
    }
    const archive = path.join(publishDir, `ShowKeyBoard_portable_v${version}_${getDateStr()}.7z`)
    if (fs.existsSync(archive)) {
        fs.unlinkSync(archive)
    }
    myExec(`"${ZIP7}" a -t7z -mx=9 "${archive}" .`, '7z', false, releaseDir)
})();

// 编译调用
function myExec(command, info, detail, cwd) {
    try {
        // 同步执行命令
        const output = execSync(command, cwd ? { cwd } : undefined);
        console.log(`编译 ${info} 成功`);
        if (detail) {
            console.log(`输出信息: ${output?.toString()}`);
        }
    } catch (error) {
        console.error(`编译 ${info} 出错: ${error.message}`);
        console.error(`命令执行过程中出现错误: ${error.stderr?.toString()}`);
        return false
    }
    return true
}

// 目录不存在则创建
function ensureDir(dir) {
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
        console.log('创建目录:', dir);
    }
    return true
}

// 复制文件（覆盖旧文件），目标目录不存在则创建
function copyFile(src, dst) {
    if (!fs.existsSync(src)) {
        console.log('文件不存在:', src);
        return false
    }
    ensureDir(path.dirname(dst));
    fs.copyFileSync(src, dst);
    console.log('替换文件:', dst);
    return true
}

// 当前日期，格式 YYMMDD，例如 261109
function getDateStr() {
    const d = new Date()
    const yy = String(d.getFullYear() % 100).padStart(2, '0')
    const mm = String(d.getMonth() + 1).padStart(2, '0')
    const dd = String(d.getDate()).padStart(2, '0')
    return `${yy}${mm}${dd}`
}

// 读取 Cargo.toml 中 [package] 段的版本号，取前两位
function getVersion(tomlFile) {
    if (!fs.existsSync(tomlFile)) {
        console.log('文件不存在:', tomlFile);
        return null
    }
    const content = fs.readFileSync(tomlFile, 'utf-8');
    // 只解析 [package] 段，避免匹配到依赖项里的 version
    const pkgSection = content.split(/^\[/m).find(s => s.startsWith('package]')) || '';
    const match = pkgSection.match(/^version\s*=\s*"([^"]+)"/m);
    if (!match) {
        console.log('未在 [package] 段找到 version:', tomlFile);
        return null
    }
    const version = match[1].split('.').slice(0, 2).join('.');
    console.log('Version:', version)
    return version
}

// 替换文件中的版本号为指定版本
function replaceVer(filePath, isGbk, ver, RegExps) {
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