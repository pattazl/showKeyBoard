import * as path from 'path'
import { window, ProgressLocation } from 'vscode'
import downloadCore from "./downloadcore.js";
import {
    getImages, escapeStringRegexp, logger,
    mdFile, localFolder, rename,
    getAutoPath, saveFile, localCheck, 
    timeoutPromise,dlTimeout,regOfImage
} from './common'
import { getLang } from './lang.js';
// 主要内部变量
//var downThread = 1;

export async function download() // ,thread:number
{
    if (!localCheck()) {
        return;
    }
    let fileObj = getImages();
    let fileArr = fileObj.net;
    let content = fileObj.content;
    //downThread = thread;
    // 对网络图片去重，不必每次下载
    let set = new Set();
    fileArr.forEach((item) => set.add(item));
    let downArr: string[] = Array.from(set) as string[];
    let count = 0, len = downArr.length;
    let successCount = 0;

    // 一直等着下载完毕，超时100秒
    let rres:any;
    var p = new Promise((resolve,reject) => {
        rres = resolve;
    });
    window.withProgress({ title: getLang('dling'), location: ProgressLocation.Notification }, async (progress, token) => {
        for (let file of downArr) {
            count++;
            logger.info(`downloading [${file}], ${count}/${len}`, false);
            let fileBasename = path.basename(file)
            progress.report({ increment: count / len * 100, message: getLang('dling2',fileBasename,count,len) });
            try {
                // 此处需要配置超时，不应该在外面超时
                let res = await timeoutPromise(downloadCore(file, localFolder, rename), dlTimeout*1000 ,getLang('dltimeout',fileBasename,dlTimeout));
                let resfile = res as string;
                if (resfile == '') { continue; }
                let newfile = getAutoPath(resfile);
                // 适配图片的格式
                var reg = regOfImage(file);
                content = content.replace(reg, '![$1](' + newfile + '$2)'); // 内容替换
                successCount++;
            } catch (e) {
                console.log(e)
                logger.error( getLang('dlerror', fileBasename) );
                rres('error')
                return Promise.reject()
            }
        }
        await saveFile(content, successCount);
        rres('finish')
        return Promise.resolve()
    });
    return p;
}
