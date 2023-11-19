// https://vitejs.dev/config/

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers'
import path from 'path'
import fs from 'fs'
import  dayjs from 'dayjs'


//import  dayjs from 'dayjs';
const setSrcVersion = () => {
  const filever = './src/version.ts'
  // WebPack2 不支持 ES6
  //let verNo = parseInt(require(filever).verNo)||0;
  //const YYMMDD = require(filever).YYMMDD.toString();
  var verNo = 0,
    YYMMDD = ''
  const verPath = path.join(__dirname, filever)
  var strDate = dayjs(new Date()).format('YYMMDD') //dayjs(new Date()).format('YYYY-MM-DD HH:mm:ss');
  let strVer = fs.readFileSync(verPath).toString()
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


// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  console.log(mode)
  if (mode == 'production') {
    setSrcVersion()
  }
  return {
    resolve: {
      alias: {
        '@': path.resolve('./src'),
      },
    },
    build: {
        emptyOutDir : true,  // 清空目标文件
        outDir: '../build/release/httpdist/dist/ui' // 设置输出目录为 ui
    },
    plugins: [
      vue(),
      AutoImport({
        imports: [
          'vue',
          {
            'naive-ui': [
              'useDialog',
              'useMessage',
              'useNotification',
              'useLoadingBar'
            ]
          }
        ]
      }),
      Components({
        resolvers: [NaiveUiResolver()]
      })
    ],
    base: './',
  }
})