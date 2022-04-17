/* eslint-disable @typescript-eslint/naming-convention */
// 此文件用于导出所有以 getLang 函数方式引用的多语言包
import * as util from 'util';
import * as vscode from 'vscode';
const config = JSON.parse(process.env.VSCODE_NLS_CONFIG || '');
let locale = config['locale'] || 'en';

// 总共需要的词组清单
type langGroup = 'hello'
|'localimage'
|'netimage'
|'invaldimage'
|'removeFolderHint'
|'removeResult'
|'moveHint'
|'docAct'
|'docDirty'
|'uptSucc'
|'uptSucc2'
|'uptSucc3'
|'docSelect'
|'installPicgo'
|'dltimeout'
|'dling'
|'dling2'
|'dlerror'
|'uptimeout'
|'uping'
|'uping2'
|'uperror'
|'extension'
|'localfolder'
|'createf'
|'notf'
|'picgotry'
|'picgotryNow'
|'picgotryLater'
|'link'
|'picgofail'
|'insertTxt'
|'inserting'
|'insertEmpty'
|'uploadFail'
|'closed'
|'posChg'
|'rangChg'
|'notChg'
// 英文字典清单
let defaultLang:Record<langGroup, string>= {
    hello: 'default hello'
    ,localimage:'----local images total[%d]----'
    ,netimage:'----net images total[%d]----'
    ,invaldimage:'----invalid local images total[%d]----'
    ,removeFolderHint: "local image path is diff, must be the same, move first!"
    ,removeResult: '----removed images total[%d] to [%s]----\n%s'
    ,moveHint: 'Choose the folder the images move to'
    ,docAct: 'document not Actived!'
    ,docDirty: 'Please save doc first!'
    ,uptSucc: 'The image links[%d] in [%s] has been updated'
    ,uptSucc2: 'Images[%d] have beaen changed,but not update content,because the setting of update link'
    ,uptSucc3: 'No image link changed'
    ,docSelect: 'No image link in selected text'
    ,installPicgo: '!!Please global install picgo-cli first,exec [npm install picgo -g] in cli'
    ,dltimeout: '!!download timeout!!'
    ,dling: 'downloading'
    ,dling2: ' [%s], %d/%d'
    ,dlerror: '!download [%s] error!'
    ,uptimeout: '!!upload timeout!!'
    ,uping: 'uploading'
    ,uping2: ' [%s], %d/%d'
    ,uperror: '!upload [%s] error!msg[%s]'
    ,extension: '!please wait for extension!'
    ,localfolder: 'local Folder[%s] is not exists, will create'
    ,createf: 'create [%s] fail!!!'
    ,notf: '[%s] is not directory!!!'
    ,picgotry: 'Link succ, please restart VSCode to take effect!'
    ,picgotryNow: 'Restart'
    ,picgotryLater: 'Later'
    ,link: 'whether link Picgo?'
    ,picgofail: 'link Picgo fail!'
    ,insertTxt: 'Insert image succ: '
    ,insertEmpty: "Can't get remote image, Insert image fail"
    ,uploadFail: "Upload [%s] fail"
    ,inserting: "Inserting Image: "
    ,closed: "The active editor is closed! Fail"
    ,posChg: "Cursor position changed"
    ,rangChg: "Select Range changed"
    ,notChg: "not update link by setting"
}
// 中文需要填写的
let zhcnLang:Record<langGroup, string>= {
    hello: '你好'
    ,localimage:'----本地图片数[%d]----'
    ,netimage:'----网络图片数[%d]----'
    ,invaldimage:'----本地无效图片共[%d]----'
    ,removeFolderHint: "本地资源文件路径必须一样，可以先移动到一个目录下!"
    ,removeResult: '----移动图片总共[%d] 到 [%s]----\n%s'
    ,moveHint: '请选择图片移动的目标目录'
    ,docAct: '文件未打开！'
    ,docDirty: '请先保存文件！'
    ,uptSucc: '已经更新图片链接[%d]个,在[%s]中'
    ,uptSucc2: '图片已经改变[%d]个,因update link设置,不更新内容'
    ,uptSucc3: '没有链接变化，无需更新'
    ,docSelect: '选中的文本中没有本地图片'
    ,installPicgo: '请先全局安装和配置PicGo-Cli,命令行执行[npm install picgo -g]'
    ,dltimeout: '!!下载超时!!'
    ,dling: '下载中'
    ,dling2: '当前 [%s], %d/%d'
    ,dlerror: '!下载 [%s] 异常!'
    ,uptimeout: '!!上传超时!!'
    ,uping: '上传中'
    ,uping2: '当前 [%s], %d/%d'
    ,uperror: '!上传[%s]失败!原因[%s]'
    ,extension: '请等待插件初始化完成'
    ,localfolder: '本地文件夹[%s]不存在，将创建'
    ,createf: '创建本地文件夹[%s]失败!'
    ,notf: `[%s]不是目录!`
    ,picgotry: '关联成功,请重启VSCode后使用'
    ,picgotryNow: '立刻重启'
    ,picgotryLater: '稍后重启'
    ,link: '是否关联Picgo?'
    ,picgofail: '关联Picgo失败'
    ,insertTxt: '成功插入图片 '
    ,insertEmpty: '未获取到远程图片，插入失败'
    ,uploadFail: "上传 [%s] 失败"
    ,inserting: "插入图片: "
    ,closed: "编辑器已经关闭，操作失败"
    ,posChg: "光标位置已改变"
    ,rangChg: "选择范围已经改变"
    ,notChg: "因设置不更新内容"
}
// 语言包汇总
let lang = {
    'en': defaultLang,
    'zh-cn': zhcnLang
};
// 通过此方法获取信息
export function getLang(key: langGroup, ...msg:any) {
    let local = lang[locale][key];
    if(msg!=null)
    {
        return util.format(local,...msg);
    }else{
        return local;
    }
    
}

