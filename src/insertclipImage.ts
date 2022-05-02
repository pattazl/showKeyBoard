import { getClipboardImage } from './getClipboardImage'
import * as path from 'path'
import * as fs from 'fs-extra'
import {
  insertText, logger, mdFile, clipboardPath,
  convertPath, getAutoPath, getValidFileName
} from './common'

import { getLang } from './lang'
// 主要内部变量
// let mdFile = ''; // 需要处理的文件
// let localFolder = ''; // 新的文件夹
// let overwriteFile = false; // 是否覆盖原先的md文件
// let rename = false; // 是否对所有的图片重新命名

export async function insertClipImage() // ,thread:number
{
  // let localFolder = lf;
  // let fileObj = getImages(true); // 定位到选的内容
  let parentPath = path.dirname(mdFile);
  // 目标保存
  let targetPath = path.resolve(parentPath, convertPath(clipboardPath.trim() || ''));
  let oTargetPath = path.parse(targetPath);
  // 自动创建目录
  fs.ensureDirSync(oTargetPath.dir);
  let validPath = getValidFileName(oTargetPath.dir, oTargetPath.base);
  let imagePath = '';
  try {
    imagePath = await uploadClibBoard(validPath);
  } catch (e: any) {
    logger.error(e.toString())
    return;
  }
  imagePath = getAutoPath(imagePath);
  let content = '![](' + imagePath + ')';
  await insertText(content);
  //await saveFile(content,count);
}
interface IImgInfo {
  buffer?: Buffer
  base64Image?: string
  fileName?: string
  width?: number
  height?: number
  extname?: string
  imgUrl?: string
  [propName: string]: any
}
async function uploadClibBoard(input: string): Promise<string> {
  // upload from clipboard
  try {
    if(input == 'no image')
    {
      throw new Error('Image name[no image] is invalid!')
    }
    const imgPath = await getClipboardImage(input)
    if (imgPath === 'no image') {
      throw new Error(getLang('noCBimage'))
    } else {
      //const { output } = await this.lifecycle.start([imgPath])
      // return 'output'
      return Promise.resolve(imgPath);
    }
  } catch (e) {
    // cbEvent.emit(IBuildInEvent.FAILED, e)
    // throw e
    return Promise.reject(e);
  }
}