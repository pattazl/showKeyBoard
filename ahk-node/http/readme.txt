发布使用
npm run build
如果提示
[ERROR] Could not resolve "mock-aws-s3" 等错误
可以尝试执行

yarn add @mapbox/node-pre-gyp@^1.0.10 aws-sdk@^2.1396.0 mock-aws-s3@^4.0.2 nock@^13.3.1
或者
yarn add @mapbox/node-pre-gyp aws-sdk mock-aws-s3 nock

需要修改package.json 设置 .node文件目录