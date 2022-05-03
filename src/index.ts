import { mdCheck, showInVscode, setPara, timeoutPromise,clearMsg, logger } from './common';
import { cleanMD, analyze } from './clean';
import { download } from './download';
import { upload, upCheck } from './upload';
import { move } from './move';
import { insertClipImage } from './insertclipImage';
import { convert } from './convert';
import { getLang } from './lang';
import * as vscode from 'vscode';
/*
import * as nls from 'vscode-nls';
const localize = nls.config({
    bundleFormat: nls.BundleFormat.standalone,
    messageFormat: nls.MessageFormat.both
})();
//const localize = nls.loadMessageBundle();
// 匹配对应的 i18n\zh-cn\xxx.i18n.json 文件
// 打开新文件
// let document = vscode.window.activeTextEditor?.document;
// if(document != null)
// {
//     let pat = 'd:\\test.txt';
//     let newUri = document.uri.with({ path: pat });
//     await vscode.window.showTextDocument(newUri, { preview: false });
// }
*/

export async function vscAnalyze() {
    // vscode.window.showInformationMessage(getLang('hello'))
    analyze();
    showInVscode();
}
export async function vscClean(flag:boolean=false) {
    cleanMD(flag);
    showInVscode();
}
export async function vscDownload() {
    await download()
    showInVscode();
}
// 上传所选图片/剪切板图片
export async function vscUpload(clip:boolean=false) {
    if (!await upCheck()) {
        showInVscode();
        return;
    }
    await upload(clip);
    showInVscode();
}
// 插入剪切板图片
export async function vscInsertClip(clip:boolean=false) {
    await insertClipImage();
    showInVscode();
}
// 根据参数，转变图片格式
export async function vscConvertImageFormat() {
    await convert()
    showInVscode();
}


// 初始化参数，参数保存于 common模块中
export function initPara() {
    clearMsg();
    let extendName = 'markdown-image-manage';
    let hasBracket = vscode.workspace.getConfiguration(extendName).get('hasBracket') as string;
    let updateLink = vscode.workspace.getConfiguration(extendName).get('updateLink') as boolean;
    let skipSelectChange = vscode.workspace.getConfiguration(extendName).get('skipSelectChange') as boolean;
    let rename = vscode.workspace.getConfiguration(extendName).get('rename') as boolean;
    let remotePath: string = vscode.workspace.getConfiguration(extendName).get('remotePath') || '<filename>';
    let imageSaveFolder: string = vscode.workspace.getConfiguration(extendName).get('imageSaveFolder') || '<filename>.assets';
    let removeFolder: string = vscode.workspace.getConfiguration(extendName).get('removeFolder') || 'md-img-remove';
    let dlTimeout: number = vscode.workspace.getConfiguration(extendName).get('timeoutDownload') || 10;
    let ulTimeout: number = vscode.workspace.getConfiguration(extendName).get('timeoutUpload') || 10;
    let clipboardPath: string = vscode.workspace.getConfiguration(extendName).get('clipboardPath') || '<filename>.assets/<YYMMDDHHmmss>.png';
    let urlFormatted: boolean = vscode.workspace.getConfiguration(extendName).get('urlFormatted') || true;
    if(dlTimeout<=0) {dlTimeout =10;}
    if(ulTimeout<=0) {ulTimeout =10;}
    //const isAsync: boolean = vscode.workspace.getConfiguration().get('downloadImageInMarkdown.isAsync') as boolean;
    setPara(hasBracket, rename, updateLink,skipSelectChange, imageSaveFolder, remotePath
        , removeFolder,dlTimeout,ulTimeout,clipboardPath,urlFormatted);

    let file = vscode.window.activeTextEditor?.document.uri.fsPath || '';
    if (!mdCheck(file)) {
        showInVscode();
        return false;
    }
    return true;
}

export async function vscMove() {
    const result = await vscode.window.showOpenDialog({
        canSelectFiles: false,
        canSelectFolders: true,
        canSelectMany: false,
        openLabel: getLang('moveHint')
    });

    if (!result || result.length === 0) {
        return;
    }

    let localFolder: string = result[0].fsPath;
    console.log(`Will Move images to localFolder[${localFolder}]`)
    await move(localFolder);
    showInVscode();
}
