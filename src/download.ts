import * as path from 'path'
import { window, ProgressLocation } from 'vscode'
import downloadCore from "./downloadcore.js";
import {
    getImages, escapeStringRegexp, logger,
    mdFile, localFolder, rename,
    getAutoPath, saveFile, localCheck,showInVscode
} from './common'
// 主要内部变量
//var downThread = 1;

export async function download() // ,thread:number
{
    if (!localCheck()) {
        return;
    }
    let fileObj = getImages();
    let ofile = path.parse(mdFile);
    let fileArr = fileObj.net;
    let content = fileObj.content;
    //downThread = thread;
    // 对网络图片去重，不必每次下载
    let set = new Set();
    fileArr.forEach((item) => set.add(item));
    let downArr: string[] = Array.from(set) as string[];
    let count = 0, len = downArr.length;
    let successCount = 0;
    //startProgress('aaaaaa',0)

    window.withProgress({ title: 'downloading', location: ProgressLocation.Notification }, async (progress, token) => {
        for (let file of downArr) {
            count++;
            logger.info(`downloading [${file}], ${count}/${len}`, false);
            progress.report({ increment: count / len * 100, message: `downloading [${file}], ${count}/${len}` });
            try {
                let obj = { rename, localFolder };
                let res = await downloadCore(file, localFolder, rename);
                let resfile = res as string;
                if (resfile == '') { continue; }
                let newfile = getAutoPath(ofile.dir, resfile);
                // 适配图片的格式
                var reg = new RegExp('!\\[([^\\]]*)\\]\\(' + escapeStringRegexp(file) + '\\)', 'ig');
                content = content.replace(reg, '![$1](' + newfile + ')'); // 内容替换
                successCount++;
            } catch (e) {
                console.log(e)
                return Promise.reject()
            }
        }
        saveFile(content, successCount);
        showInVscode();
        return Promise.resolve()
        //else return Promise.reject()
    });
}

function startProgress(message: string, process: number) {

    window.withProgress({ title: message, location: ProgressLocation.Notification }, async (progress, token) => {
        progress.report({ increment: 10, message: "I am long running! - still going..." });
        let retries = 60;
        //return Promise.resolve('aaaa')
        var p = new Promise(resolve => {
            setTimeout(() => {
                resolve('0');
            }, 5000);
        });
        return p;
        //else return Promise.reject()
    });
}