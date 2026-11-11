// 根据 rust/client/Cargo.toml 的主版本号更新 .nsi，编译 release 产物并打包安装包与便携版（Windows）
// macOS 平台：将 build/release 目录打包为 DMG（入口程序 showkeyboard_client，图标 res/keyboard.ico）
// 此文件从 rust 目录运行 npm run build / npm run pack 调用：
//   npm run build            编译模式：先 cargo build --release 并替换产物，再打包
//   npm run pack  (--pack)   打包模式：跳过 cargo 编译，直接用 build/release 现有文件打包
// macOS 打包前会先用 build/releaseMac 中的同名文件替换 build/release 下的对应文件，打包结束后再换回原文件
// 平台自动识别：darwin -> DMG 打包；win32/其它 -> 原有 NSIS 安装包 + 7z 便携版流程
const { NSIS, ZIP7 } = require('./buildMain.config.js');
const fs = require('fs');
const path = require('path');
const { execSync, execFileSync } = require('child_process');
// iconv-lite 仅 Windows 的 GBK .nsi 版本替换需要，惰性加载（macOS 打包无需安装）

// ========== 计时工具 ==========
const _globalStart = Date.now();
function ts() {
    const d = new Date();
    const p = n => String(n).padStart(2, '0');
    return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}
function fmtMs(ms) {
    const s = Math.round(ms / 1000);
    if (s < 60) return `${s}s`;
    const m = Math.floor(s / 60);
    const rs = s % 60;
    if (m < 60) return `${m}m ${String(rs).padStart(2, '0')}s`;
    const h = Math.floor(m / 60);
    const rm = m % 60;
    return `${h}h ${String(rm).padStart(2, '0')}m ${String(rs).padStart(2, '0')}s`;
}
// 进程退出时打印总耗时（跨 try/finally 保证一定输出）
process.on('exit', () => {
    const total = Date.now() - _globalStart;
    console.log('');
    console.log(`========== 全部完成  开始 ${tsStart}  结束 ${ts()}  总耗时 ${fmtMs(total)} ==========`);
});
const tsStart = ts();
console.log(`========== 开始构建  ${tsStart}  ==========`);

// 平台分发
if (process.platform === 'darwin') {
    packMacDmg();
} else {
    packWindows();
}

// =====================================================================
// Windows：NSIS 安装包 + 7z 便携版（原逻辑，未改动）
// =====================================================================
function packWindows() {
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
}

// =====================================================================
// macOS：release 目录 -> ShowKeyBoard.app -> DMG
// =====================================================================
function packMacDmg() {
    const skipCargo = process.argv.includes('--pack')
    const clientDir = path.resolve(__dirname, '../client');
    const serverDir = path.resolve(__dirname, '../server');
    const cargoToml = path.join(clientDir, 'Cargo.toml');
    const releaseDir = path.resolve(__dirname, 'release');
    const distDir = path.join(releaseDir, 'httpdist', 'dist');
    const publishDir = path.resolve(__dirname, 'publish');
    const iconIco = path.resolve(__dirname, '../res/keyboard.ico');
    const appName = 'ShowKeyBoard';

    console.log(skipCargo ? '打包模式：跳过 cargo 编译' : '编译模式：编译 release 产物后打包')
    const version = getVersion(cargoToml)
    if (!version) return

    ensureDir(publishDir)

    // 1.（可选）编译 release 产物并替换到 release 目录
    if (!skipCargo) {
        if (!myExec('cargo build --release', 'client release', false, clientDir)) return
        if (!myExec('cargo build --release', 'server release', false, serverDir)) return
        if (!copyFile(path.join(clientDir, 'target', 'release', 'showkeyboard_client'),
            path.join(releaseDir, 'showkeyboard_client'))) return
        if (!copyFile(path.join(serverDir, 'target', 'release', 'showkeyboard_server'),
            path.join(distDir, 'showkeyboard_server'))) return
    }

    if (!fs.existsSync(path.join(releaseDir, 'showkeyboard_client'))) {
        console.log('入口程序不存在:', path.join(releaseDir, 'showkeyboard_client'));
        return
    }
    if (!fs.existsSync(iconIco)) {
        console.log('图标不存在:', iconIco);
        return
    }

    // 2. 用 build/releaseMac 中的 macOS 专用配置替换 release 下的对应文件
    //    restoreMacOverrides：打包结束后把 release 中原文件换回来
    const restoreMacOverrides = applyMacOverrides(releaseDir)
    if (!restoreMacOverrides) return

    // 3. 临时目录
    const tmpRoot = fs.mkdtempSync(path.join(require('os').tmpdir(), 'skb_dmg.'));
    const stageDir = path.join(tmpRoot, 'stage');
    const appDir = path.join(stageDir, `${appName}.app`);
    const macosDir = path.join(appDir, 'Contents', 'MacOS');
    const resDir = path.join(appDir, 'Contents', 'Resources');
    fs.mkdirSync(macosDir, { recursive: true });
    fs.mkdirSync(resDir, { recursive: true });

    try {
        // 4. 复制 release 全部内容到 bundle 内（客户端以 exe 所在目录为工作目录，
        //    httpdist/ini 等相对路径照常工作）
        console.log('复制 release 目录到 app bundle');
        fs.cpSync(releaseDir, macosDir, { recursive: true });
        // 运行期/用户数据文件不带入安装包：首次运行由客户端/服务端迁移到
        // ~/Documents/ShowKeyBoard（showKeyBoard.ini / KeyList.txt 保留在包内作为初始模板）
        for (const f of ['kb.pid', 'kbserver.pid', 'lastRecord.json', 'updateTime.txt']) {
            fs.rmSync(path.join(macosDir, f), { force: true });
        }
        for (const d of ['dbs', 'backup', 'share', 'uploads', 'httpdist/dist/dbs', 'httpdist/dist/uploads']) {
            fs.rmSync(path.join(macosDir, d), { recursive: true, force: true });
        }
        chmodX(path.join(macosDir, 'showkeyboard_client'));
        const serverBin = path.join(macosDir, 'httpdist', 'dist', 'showkeyboard_server');
        if (fs.existsSync(serverBin)) chmodX(serverBin);

        // 5. 图标：ico -> png -> iconset(10 档尺寸) -> icns
        console.log('转换图标:', iconIco);
        const icns = path.join(resDir, 'AppIcon.icns');
        if (!convertIcns(iconIco, icns, tmpRoot)) return;

        // 6. Info.plist（LSUIElement：不在 Dock 显示图标，与客户端 Accessory 策略一致）
        fs.writeFileSync(path.join(appDir, 'Contents', 'Info.plist'), `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>${appName}</string>
    <key>CFBundleDisplayName</key><string>${appName}</string>
    <key>CFBundleIdentifier</key><string>com.pattazl.showkeyboard</string>
    <key>CFBundleVersion</key><string>${version}</string>
    <key>CFBundleShortVersionString</key><string>${version}</string>
    <key>CFBundleExecutable</key><string>showkeyboard_client</string>
    <key>CFBundleIconFile</key><string>AppIcon</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>LSUIElement</key><true/>
</dict>
</plist>
`);

        // 7. ad-hoc 签名（本机运行免网闸；不做公证）
        if (!run('codesign', ['--force', '--deep', '--sign', '-', appDir], null, 'codesign 签名')) {
            console.log('!!! codesign 失败（可忽略，未签名也可本机运行）');
        }

        // 8. DMG：stage 目录 + Applications 快捷方式（拖拽安装）
        fs.symlinkSync('/Applications', path.join(stageDir, 'Applications'));
        const dmgName = `${appName}_macos_v${version}_${getDateStr()}.dmg`;
        const dmgPath = path.join(publishDir, dmgName);
        fs.rmSync(dmgPath, { force: true });
        if (!run('hdiutil', [
            'create',
            '-volname', `${appName} v${version}`,
            '-srcfolder', stageDir,
            '-fs', 'HFS+',
            '-format', 'UDZO',
            '-ov',
            dmgPath,
        ], null, 'hdiutil 打包 DMG')) return;

        const sizeKb = Math.round(fs.statSync(dmgPath).size / 1024);
        console.log('');
        console.log('打包完成:');
        console.log(`  版本: v${version}`);
        console.log(`  产物: ${dmgPath}`);
        console.log(`  大小: ${sizeKb >= 1024 ? (sizeKb / 1024).toFixed(1) + 'M' : sizeKb + 'K'}`);
    } finally {
        fs.rmSync(tmpRoot, { recursive: true, force: true });
        // 打包结束（含异常/提前返回）后恢复 release 中原有文件
        restoreMacOverrides();
    }
}

// ico -> icns：sips 转 png，再按 macOS 图标规范生成 10 档尺寸，iconutil 合成 icns
function convertIcns(icoPath, outIcns, tmpDir) {
    const basePng = path.join(tmpDir, 'icon_base.png');
    if (!run('sips', ['-s', 'format', 'png', icoPath, '--out', basePng], null, 'sips ico->png')) return false;
    const iconset = path.join(tmpDir, 'AppIcon.iconset');
    fs.mkdirSync(iconset, { recursive: true });
    // [尺寸, 文件名]（@2x 为 Retina 高清档）
    const sizes = [
        [16, 'icon_16x16.png'], [32, 'icon_16x16@2x.png'],
        [32, 'icon_32x32.png'], [64, 'icon_32x32@2x.png'],
        [128, 'icon_128x128.png'], [256, 'icon_128x128@2x.png'],
        [256, 'icon_256x256.png'], [512, 'icon_256x256@2x.png'],
        [512, 'icon_512x512.png'], [1024, 'icon_512x512@2x.png'],
    ];
    for (const [size, name] of sizes) {
        if (!run('sips', ['-z', String(size), String(size), basePng, '--out', path.join(iconset, name)], null, `sips ${size}px`)) {
            return false;
        }
    }
    return run('iconutil', ['-c', 'icns', iconset, '-o', outIcns], null, 'iconutil icns');
}

// 可执行权限
function chmodX(file) {
    try {
        fs.chmodSync(file, 0o755);
    } catch (e) {
        console.log('chmod 失败:', file, e.message);
    }
}

// 同步执行外部命令（参数数组形式，避免引号转义问题）
function run(cmd, args, cwd, info) {
    const stepStart = Date.now();
    const label = info || cmd;
    console.log(`[${ts()}] ▶ ${label}`);
    try {
        execFileSync(cmd, args, { cwd: cwd || undefined, stdio: ['ignore', 'pipe', 'pipe'] });
        console.log(`[${ts()}] ✔ ${label} 完成 (${fmtMs(Date.now() - stepStart)})`);
        return true;
    } catch (e) {
        console.error(`[${ts()}] ✘ ${label} 失败 (${fmtMs(Date.now() - stepStart)}): ${e.message}`);
        if (e.stderr) console.error(e.stderr.toString());
        return false;
    }
}

// 编译调用
function myExec(command, info, detail, cwd) {
    const stepStart = Date.now();
    const label = info || command;
    console.log(`[${ts()}] ▶ ${label}`);
    try {
        const output = execSync(command, cwd ? { cwd } : undefined);
        console.log(`[${ts()}] ✔ ${label} 完成 (${fmtMs(Date.now() - stepStart)})`);
        if (detail) {
            console.log(`输出信息: ${output?.toString()}`);
        }
    } catch (error) {
        console.error(`[${ts()}] ✘ ${label} 出错 (${fmtMs(Date.now() - stepStart)}): ${error.message}`);
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

// macOS 专用配置：把 build/releaseMac 下的文件按文件名替换 release 中对应文件
// 返回恢复函数（打包结束后调用，把 release 原文件换回来）；返回 false 表示替换失败
function applyMacOverrides(releaseDir) {
    const macDir = path.resolve(__dirname, 'releaseMac');
    if (!fs.existsSync(macDir)) {
        console.log('macOS 配置目录不存在，跳过:', macDir);
        return () => { };
    }
    // 备份原文件内容，用于打包后恢复
    const backups = [];
    const restore = () => {
        for (const { file, data } of backups) {
            fs.writeFileSync(file, data);
            console.log('恢复原配置文件:', file);
        }
        backups.length = 0;
    };
    const names = fs.readdirSync(macDir, { withFileTypes: true })
        .filter(d => d.isFile() && !d.name.startsWith('.'))
        .map(d => d.name);
    for (const name of names) {
        const src = path.join(macDir, name);
        const targets = findFilesByName(releaseDir, name);
        if (!targets.length) {
            console.log('release 中未找到对应的文件:', name);
            restore();
            return false;
        }
        for (const dst of targets) {
            backups.push({ file: dst, data: fs.readFileSync(dst) });
            fs.copyFileSync(src, dst);
            console.log('替换 macOS 配置文件:', dst);
        }
    }
    return restore;
}

// 递归查找目录下指定文件名的所有文件
function findFilesByName(dir, name, result = []) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const p = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            findFilesByName(p, name, result);
        } else if (entry.name === name) {
            result.push(p);
        }
    }
    return result;
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
        const iconv = isGbk ? require('iconv-lite') : null
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
                // 二进制写入原文件
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
