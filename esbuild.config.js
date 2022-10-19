const fs = require('fs');
const path = require('path');
let outDir = 'out'
const esbuildConfig = () => require('esbuild').buildSync({
    entryPoints: [
        'src/extension.ts',
        //'index.dev.ts',
    ],
    bundle: true,
    loader: { ".ts": "ts" },
    outdir: outDir,
    platform: "node",
    target: "ES2020",
    minify: true,
    external: ['vscode', 'electron'],
})
//  清理 outDir 目录不需要的文件
let arr = fs.readdirSync(outDir);
arr.forEach((val, idx) => {
    let currPath = path.join(outDir, val)
    let ph = fs.statSync(currPath);
    if (ph.isFile()) {
        //console.log(`${currPath} 是文件`);
        fs.unlinkSync(currPath)
    }
})
esbuildConfig();