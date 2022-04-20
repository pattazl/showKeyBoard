import * as fs from "fs-extra";
import * as https from "https";
import * as http from "http";
import * as path from "path";
import { URL } from "url";
import { logger, newName ,getValidFileName } from './common';
// 伪装成浏览器
// eslint-disable-next-line @typescript-eslint/naming-convention
const headers = { "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36" };
const options = { strictSSL: false, headers };
// 同步下载方法封装
let rename = false;
async function download(url: string, dest: string, options:{}) {
  const uri = new URL(url);
  let filename = path.basename(url); // 获取基本的文件名
  const pkg = url.toLowerCase().startsWith("https:") ? https : http;
  return await new Promise((resolve, reject) => {
    pkg.get(uri.href, options).on("response", (res) => {
      //console.log("res", res.statusCode, res.headers);
      //const len = parseInt(res.headers["content-length"], 10);
      fs.ensureDirSync(dest);
      if (rename) {
        filename = newName() + path.extname(filename); // 36 进制
      }

      let filePath = getValidFileName(dest, filename);
      if (filePath == '') {
        reject('get file path fail!');
        return;
      }
      if (res.statusCode === 200) {
        const file = fs.createWriteStream(filePath);
        let num = 0;
        res
          .on("data", (chunk:any) => {
            num = chunk.length + num;
            //console.log(chunk.length, num, chunk);
          })
          .on("end", () => {
            resolve(filePath);
          })
          .on("error", (err:any) => {
            reject(err);
          })
          .pipe(file);
      } else if (res.statusCode === 302 || res.statusCode === 301) {
        // Recursively follow redirects, only a 200 will resolve.
        download(res.headers.location||'', dest, options).then((val) =>
          resolve(val)
        );
      } else {
        reject({
          code: res.statusCode,
          message: res.statusMessage,
        });
      }
    });
  });
}

export default async (url: any, dest: any, re: boolean) => {
  try {
    rename = re;
    const filePath = await download(url, dest, options);
    logger.success(filePath + "is downloaded",false);
    // 等下
    await new Promise(resolve => {
        setTimeout(() => {
            resolve('0');
        }, 2000);
    })
    return filePath;
  } catch (e) {
    logger.error( 'Download Error:'+JSON.stringify(e) )
    return '';
  }
};
