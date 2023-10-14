const fs = require('fs');
const path = require('path');
let outDir = '../httpdist/dist'
const esbuildConfig = () => require('esbuild').buildSync({
    entryPoints: [
        'src/server.js',
    ],
    bundle: true,
    sourcemap:false,
    loader: { ".ts": "ts" ,".js": "js" },
    outdir: outDir,
    platform: "node",
    external: ['nock', 'mock-aws-s3', 'aws-sdk'], // [], //
    target: [
      'node10',
    ],
	//outfile: 'dist/onetrack.js',
    minify: true,
})
// 版本显示
const dayjs = require('dayjs')
const setSrcVersion = () => {
    // 读取 package.json 的版本为主版本
    const packageJsonPath = path.resolve(__dirname, 'package.json');
    const packageJsonContent = fs.readFileSync(packageJsonPath, 'utf-8');
    // 解析 JSON
    const packageJson = JSON.parse(packageJsonContent);
    // 版本文件需要动态修改内容
    const filever = './src/version.js'
    var verNo = 0, YYMMDD = '';
    const verPath = path.join(__dirname, filever)
    var strDate = dayjs(new Date()).format('YYMMDD') //dayjs(new Date()).format('YYYY-MM-DD HH:mm:ss');
    let strVer = fs.readFileSync(verPath).toString()

    strVer = strVer.replace(/const mainVersion = '(.*)'/, function (a, b) {
        return `const mainVersion = '${packageJson.version}'`
    })
    strVer = strVer.replace(/const replaceYYMMDD = '(.*)'/, function (a, b) {
        YYMMDD = b
        return `const replaceYYMMDD = '${strDate}'`
    })
    strVer = strVer.replace(/const replaceVerNo = '(.*)'/, function (a, b) {
        verNo = parseInt(b) || 0
        if (strDate === YYMMDD) {
            verNo++ // 如果日期一样则进行版本号累加，
        } else {
            verNo = 0 // 如果日期不一样则直接复位0
        }
        return `const replaceVerNo = '${verNo}'`
    })
    fs.writeFileSync(verPath, strVer)
}
setSrcVersion();
esbuildConfig();
// 将 src/res 目录的内容编译到 /res 中