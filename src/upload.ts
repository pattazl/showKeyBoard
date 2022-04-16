/* eslint-disable @typescript-eslint/naming-convention */
import { window, ProgressLocation } from 'vscode'
import * as path from 'path';
import {
    getImages, escapeStringRegexp, logger, saveFile, remotePath,
    rename, convertPath, insertText
} from './common';
import { getLang } from './lang';
let {execSync,exec} = require('child_process');
import {promisify} from 'util';
import  * as fs from 'fs';
// 主要内部变量
//var downThread = 1;
let myPicgo: any = null; // picgo对象
let remote = ''; // 是否路径中不增加md文件名的文件夹，默认会自动增加文件夹以将不同md文件的图片分离开

let myExec = promisify(exec);
async function installPicgo() {
    //logger.info('check picgo installation...')
    //console.log(__dirname)
    // console.log(__filename)
    let runPath = path.resolve(__dirname +'/../'); // 上一级目录
    console.log(runPath)
    try {
        execSync('picgo -v', { cwd: runPath });  // 尝试执行 PicGo命令
    } catch (e) {
        logger.error(getLang('installPicgo'));
        return 'error';
    }
    let rres: any;
    var p = new Promise((resolve, reject) => {
        rres = resolve;
    });
    let answer = await window.showInformationMessage(getLang('link'), "YES", "NO")
    if (answer === "YES") {
        window.withProgress({ title: 'linking picgo...', location: ProgressLocation.Notification }, async (progress, token) => {
            let count =0;
            setInterval(function(){
                count ++;
                progress.report({ increment: count%100, message: "..." });
            },100)
            try{
                // 先改名 package.json
                fs.renameSync(runPath + '/package.json',runPath + '/ori-package.json');
                // copy package.json
                fs.copyFileSync(runPath + '/package-linkpicgo.json', runPath + '/package.json')
                await myExec('npm link picgo', { cwd: runPath });  // 通过命令进行关联 npm link picgo --omit dev ping 127.0.0.1 -n 7
            }catch(e)
            {
                rres('error')
                logger.error(e as any);
            }finally{
                // 原来配置文件还原
                fs.unlinkSync(runPath + '/package.json');
                fs.renameSync(runPath + '/ori-package.json',runPath + '/package.json');
            }
            rres('ok')
            return Promise.resolve();
        }).then(()=>{});
    } else {
        rres('error')
    }
    return p;
}
export async function upCheck() {
    try {
        const { PicGo } = require('picgo');
        myPicgo = PicGo;
    } catch (e) {
        let res = await installPicgo()
        if( res == 'ok')
        {
            logger.warn(getLang('picgotry'))
        }else{
            logger.error(getLang('picgofail'))
        }
        return false;
    }
    // 需要处理的文件
    // 对MD的结构化
    remote = convertPath(remotePath)
    return true;
}

export async function upload(clipBoard: boolean = false) // ,thread:number
{
    const picgo1 = new myPicgo(); // 将使用默认的配置文件：~/.picgo/config.json
    picgo1.on('beforeUpload', (ctx: any) => {
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
    let fileMapping: {};
    let fileArr;
    let content: string;
    if (clipBoard) {
        fileArr = [''];
    } else {
        let fileObj = getImages(true); // 根据选择的内容上传
        fileArr = fileObj.local; // 本地文件上传
        fileMapping = fileObj.mapping; // 本地原始信息
        content = fileObj.content;
        if (fileArr.length == 0) {
            logger.error(getLang('docSelect'))
            return;
        }
    }
    //downThread = thread;
    // 对网络图片去重，不必每次下载
    let set = new Set();
    fileArr.forEach((item) => set.add(item));
    let upArr: string[] = Array.from(set) as string[];
    let count = 0, len = upArr.length;
    let successCount = 0;

    // 一直等着下载完毕，超时100秒
    let rres: any;
    var p = new Promise((resolve, reject) => {
        rres = resolve;
    });
    window.withProgress({ title: getLang('uping'), location: ProgressLocation.Notification }, async (progress, token) => {
        for (let file of upArr) {
            count++;
            if (clipBoard) {
                file = 'clipboard';
            }
            logger.info(`uploading [${file}], ${count}/${len}`, false);
            progress.report({ increment: count / len * 100, message: getLang('uping2', path.basename(file), count, len) });
            try {
                let upList: string[] = [];
                if (!clipBoard) {
                    upList = [file];
                }
                let netFile = await picgo1.upload(upList);// 一次上传一个
                // 成功上传返回结果
                if (netFile.length > 0) {
                    let first = netFile[0];
                    if (first.url == null) { continue; }
                    if (clipBoard) {
                        content = '![](' + first.url + ')';
                    } else {
                        // 适配图片的格式
                        var reg = new RegExp('!\\[([^\\]]*)\\]\\(' + escapeStringRegexp(fileMapping[file]) + '\\)', 'ig');
                        content = content.replace(reg, '![$1](' + first.url + ')'); // 内容替换
                    }
                    successCount++;
                }
            } catch (e) {
                console.log(e);
                logger.error(getLang('uperror', path.basename(file), (e as any).message || ''));
                rres('error')
                return Promise.resolve()
            }
        }
        if (clipBoard) {
            insertText(content);
        } else {
            saveFile(content, successCount, true);
        }
        rres('finish')
        return Promise.resolve()
        //else return Promise.reject()
    });
    return p;
}

