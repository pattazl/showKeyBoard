let outDir = 'dist'
const sqlite3Binary = require.resolve("sqlite3/package.json");
const sqlite3Dir = require("path").dirname(sqlite3Binary);
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
      'node16',
    ],
	//outfile: 'dist/onetrack.js',
    minify: true,
    nodePaths: [sqlite3Dir],
})
esbuildConfig();
// 将 src/res 目录的内容编译到 /res 中