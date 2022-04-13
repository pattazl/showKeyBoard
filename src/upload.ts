import * as fs from 'fs';
import * as path from 'path';
import { getImages, escapeStringRegexp, logger, saveFile, remotePath,rename,convertPath} from './common';
// 主要内部变量
//var downThread = 1;
let myPicgo:any= null; // picgo对象
let remote = ''; // 是否路径中不增加md文件名的文件夹，默认会自动增加文件夹以将不同md文件的图片分离开

export function upCheck() {
    try {
        // eslint-disable-next-line @typescript-eslint/naming-convention
        const { PicGo } = require('picgo');
        myPicgo = PicGo;
    } catch (e) {
        logger.error('please install picgo first and exec ( L ) commander ');
        return false;
    }
    // 需要处理的文件
    // 对MD的结构化
    remote = convertPath(remotePath)
    return true;
}
export function linkPicgo()
{
    console.log('check picgo installation...');
    let runPath = path.dirname(require?.main?.filename||'');
    let execSync = require('child_process').execSync;
    try {
        execSync('picgo -v', { cwd: runPath });  // 尝试执行 PicGo命令
    } catch (e) {
        logger.error('!!Have not install global picgo, please run "npm install picgo -g" ');
        return;
    }
    console.log('linking picgo...');
    try {
        execSync('npm link picgo', { cwd: runPath });  // 通过命令进行关联
    } catch (e) {
        logger.error('!!Link error!!');
        return;
    }
    logger.success('link picgo succ');
}

export async function upload() // ,thread:number
{
    const picgo1 = new myPicgo(); // 将使用默认的配置文件：~/.picgo/config.json
    picgo1.on('beforeUpload', (ctx:any) => {
        let fileName = ctx.output[0].fileName;
        let upFile = path.parse(fileName);
        if (rename) {
            // 36 进制重命名上传后的文件
            fileName = path.join(upFile.dir, new Date().getTime().toString(36) + upFile.ext);
        }
        if (remote != '') {
            // 需要添加 md名的目录
            ctx.output[0].fileName = `${remote}/` + fileName;
        }
        // console.log(ctx.output) // [{ base64Image, fileName, width, height, extname }]
    });
    let fileObj = getImages();
    let fileArr = fileObj.local; // 本地文件上传
    let fileMapping = fileObj.mapping; // 本地原始信息
    let content = fileObj.content;
    //downThread = thread;
    // 对网络图片去重，不必每次下载
    let set = new Set();
    fileArr.forEach((item) => set.add(item));
    let upArr: string[] = Array.from(set) as string[];
    let count = 0, len = upArr.length;
    let successCount = 0;
    for (let file of upArr) {
        count++;
        logger.info(`uploading [${file}], ${count}/${len}`);
        try {
            let netFile = await picgo1.upload([file]);// 一次上传一个
            // 成功上传返回结果
            if (netFile.length > 0) {
                let first = netFile[0];
                if(first.url==null){continue;}
                // 适配图片的格式
                var reg = new RegExp('!\\[([^\\]]*)\\]\\(' + escapeStringRegexp(fileMapping[file]) + '\\)', 'ig');
                content = content.replace(reg, '![$1](' + first.url + ')'); // 内容替换
                successCount++;
            }
        } catch (e) {
            console.log(e);
        }
    }
    saveFile(content, successCount);
}

