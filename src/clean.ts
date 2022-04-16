import * as fs from 'fs';
import * as path from 'path';
import { getImages, logger, removeFolder, escapeStringRegexp, saveFile } from './common';
import { getLang } from './lang';

const extArr = ['.png', '.jpg', '.bmp', '.gif', '.jpeg', '.ico', '.tga', '.rle', '.tif', '.cur', '.ani', '.iff']; // 支持的图片格式


export function cleanMD(linkFlag: boolean = false) {
    if (linkFlag) {
        cleanInvalidLinks()
    } else {
        analyzeCore(true);
    }
}
export function analyze() {
    analyzeCore();
}
function cleanInvalidLinks() {
    try {
        var obj = getImages();
        if (obj.content == '') { return; }
        // 循环替换失效的链接
        let content = obj.content;
        let count = 0;
        for (let file of obj.invalid) {
            logger.info(`clean link [${file}]`, false);
            var reg = new RegExp('!\\[([^\\]]*)\\]\\(' + escapeStringRegexp(file) + '\\)', 'ig');
            content = content.replace(reg, ''); // 内容替换
            count++;
        }
        saveFile(content, count);
    } catch (e: any) {
        logger.error(e.message);
    }
}
function analyzeCore(flag: boolean = false) {
    try {
        var obj = getImages();
        if (obj.content == '') { return; }
        if (flag) {
            doFile(obj.local);
        } else {
            logger.info(getLang('localimage', obj.local.length));
            logger.info(`${obj.local.join('\n')}`);
            logger.info(getLang('netimage', obj.net.length));
            logger.info(`${obj.net.join('\n')}`);
        }
    } catch (e: any) {
        logger.error(e.message);
    }
}
function doFile(picArr: string[]) {
    if (picArr.length === 0) { return; }
    // 必须要每个文件的文件夹一样，这样可以确定为是同一个文件夹
    var parentPath = '';
    var mdFile: string[] = [];
    for (var i = 0; i < picArr.length; i++) {
        var currpath = path.dirname(picArr[i]);
        if (parentPath === '') { parentPath = currpath; }
        if (parentPath !== currpath) {
            logger.error(getLang('removeFolderHint')); // 图片文件夹不一样！
            return;
        }
        mdFile.push(path.basename(picArr[i]));
    }
    // 循环走文件，如果是图片，但是不在 mdFile 中，则进行处理
    var removeArr: string[] = [];
    // 获取 prePath 目录下所有文件
    fs.readdirSync(parentPath).forEach(function (name) {
        var filePath = path.join(parentPath, name);
        var stat = fs.statSync(filePath);
        if (stat.isFile()) {
            var orifile = name.toLowerCase();
            var extName = path.extname(orifile);
            if (extArr.indexOf(extName) > -1) // 必须是图片
            {
                if (mdFile.indexOf(orifile) === -1) {
                    // 之前没有改名过
                    let removed = path.join(parentPath, removeFolder);
                    if (!fs.existsSync(removed)) {
                        fs.mkdirSync(removed);
                    }
                    // 移动目录
                    fs.renameSync(filePath, path.join(removed, name));
                    removeArr.push(orifile);
                }
            }
        } else if (stat.isDirectory()) {
        }
    });
    logger.info(getLang('removeResult', removeArr.length, removeFolder, removeArr.join('\n')));
}
