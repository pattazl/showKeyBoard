let outDir = 'dist'
const esbuildConfig = () => require('esbuild').buildSync({
    entryPoints: [
        'src/setting.js',
    ],
    bundle: true,
    sourcemap:false,
    loader: { ".ts": "ts"  },
    outdir: outDir,
    platform: "browser",
    target: [
      'chrome58',
      'firefox57',
      'safari11',
    ],
	//outfile: 'dist/onetrack.js',
    minify: true,
})
esbuildConfig();
// 将 src/res 目录的内容编译到 /res 中