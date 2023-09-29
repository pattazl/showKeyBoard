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
esbuildConfig();
// 将 src/res 目录的内容编译到 /res 中