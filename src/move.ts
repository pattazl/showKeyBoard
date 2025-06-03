import * as fs from 'fs'
import * as path from 'path'
import { getImages,escapeStringRegexp,logger,mdFile,rename,
    newName ,getAutoPath,saveFile,getValidFileName,regOfImage} from './common'
// 主要内部变量
// let mdFile = ''; // 需要处理的文件
// let localFolder = ''; // 新的文件夹
// let overwriteFile = false; // 是否覆盖原先的md文件
// let rename = false; // 是否对所有的图片重新命名

export async function move(lf:string,copyFlag:boolean) // ,thread:number
{
    let localFolder = lf;
    let fileObj = getImages();
    if(fileObj.content == '')
    {
        return '';
    }
    let fileArr = fileObj.local; // 本地文件上传
    let fileMapping = fileObj.mapping; // 本地原始信息
    let content = fileObj.content;
    //downThread = thread;
    // 对网络图片去重，不必每次下载
    let set = new Set(); 
    fileArr.forEach((item)=> set.add(item)); 
    let uniArr:string[] = Array.from(set) as string[];
    let count=0,len = uniArr.length;
    for(let file of uniArr)
    {
        let newFileName = '';
        // 转移到目标路径 
        let imageFile = path.parse(file);
        if(rename)
        {
            //文件重命名
            newFileName = newName(imageFile.name)+ imageFile.ext;
        }else{
            // 仅仅更换目录
            newFileName = imageFile.base;
        }
        let newFile = await getValidFileName(localFolder,newFileName);
        if( newFile == ''){
            logger.error(`get new image file name[${newFile}] fail!`);
            return '';
        }
        logger.info(`[${file}] ${copyFlag?'copy':'move'} to [${newFile}], ${count}/${len}`,false);
        try{
            if(copyFlag)
            {
                fs.copyFileSync(file,newFile);
            }else{
                fs.renameSync(file,newFile);
            }
            var reg = regOfImage(fileMapping[file]);
            content =  content.replace(reg,'![$1]('+ getAutoPath( newFile) +'$2)'); // 内容替换
            count++;
        }catch(e)
        {
            logger.error('move/copy error:');
            console.log(e);
        }
    }
    await saveFile(content,count);
}

