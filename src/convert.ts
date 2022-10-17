import * as fs from 'fs'
import * as path from 'path'
import {
    getImages, escapeStringRegexp, logger,
    getAutoPath, saveFile, myEncodeURI,switchPath
} from './common'
import { getLang } from './lang';

export async function convert(formatFlag: boolean = true) // ,thread:number
{
    let fileMapping: {};
    let fileArr;
    let content = '';

    let fileObj = getImages(true); // 根据选择的内容上传
    fileArr = fileObj.local; // 本地文件上传
    fileMapping = fileObj.mapping; // 本地原始信息
    content = fileObj.content;
    if (fileArr.length == 0) {
        logger.error(getLang('docSelect'))
        return;
    }
    let set = new Set();
    fileArr.forEach((item) => set.add(item));
    let uniArr: string[] = Array.from(set) as string[];
    let count = 0, len = uniArr.length;
    enum PathFlag {unknown, isAbs, isRelative}
    let pathFlag:PathFlag = PathFlag.unknown

    for (let file of uniArr) {
        // 判断原来的格式是否为转义的
        let oriFile = fileMapping[file];
        let newFile = ''
        let flag = false
        if (formatFlag) {
            if (decodeURI(oriFile) == oriFile) {
                flag = true
            }
            newFile = myEncodeURI(oriFile, flag);
            logger.info(`format[${oriFile}] convert to [${newFile}], ${count + 1}/${len}`, false);
        } else {
            // 根据第一个路径格式来判断，相对和绝对路径转换
            if(pathFlag ==PathFlag.unknown){
                if (path.isAbsolute(oriFile) ) {
                    pathFlag = PathFlag.isAbs
                }else{
                    pathFlag = PathFlag.isRelative
                }
            }
            newFile = switchPath(oriFile,pathFlag == PathFlag.isAbs);
            logger.info(`path[${oriFile}] convert to [${newFile}], ${count + 1}/${len}`, false);
        }
        try {
            var reg = new RegExp('!\\[([^\\]]*)\\]\\(' + escapeStringRegexp(oriFile) + '\\)', 'ig');
            content = content.replace(reg, '![$1](' + newFile + ')'); // 内容替换
            count++;
        } catch (e) {
            logger.error('convert error:');
            console.log(e);
        }
    }
    await saveFile(content, count, true);
}

