let outDir = 'dist'
const esbuildConfig = () => require('esbuild').buildSync({
    entryPoints: [
        'src/server.js',
    ],
    bundle: true,
    sourcemap:true,
    loader: { ".ts": "ts"  },
    outdir: outDir,
    platform: "node",
    external: [],
    target: [
      'node8',
    ],
	//outfile: 'dist/onetrack.js',
    //minify: true,
})
esbuildConfig();
// 将 src/res 目录的内容编译到 /res 中