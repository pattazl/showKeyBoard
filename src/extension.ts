// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import {vscAnalyze,initPara,vscClean,vscMove,vscDownload, vscUpload,vscInsertClip,vscConvertImageFormat}  from './index';
// this method is called when your extension is activated
// your extension is activated the very first time the command is executed

export function activate(context: vscode.ExtensionContext) {
	// Use the console to output diagnostic information (console.log) and errors (console.error)
	// This line of code will only be executed once when your extension is activated
	console.log('Congratulations, extension "markdown-image-manage" is now active!');

	// The command has been defined in the package.json file
	// Now provide the implementation of the command with registerCommand
	// The commandId parameter must match the command field in package.json
	/*let disposable = vscode.commands.registerCommand("markdown-image-manage.helloWorld", async () => {
		let answer = await vscode.window.showInformationMessage("How was your day ?", "good", "bad",)
		if (answer === "bad") {
			vscode.window.showErrorMessage("sorry to hear it", "1", "2","3","4")
		} else {
			console.log({ answer })
			vscode.window.showWarningMessage("sorry to hear it")
		}
	})*/
	//let obj2 = vscode.workspace.getConfiguration('markdown-image-manage');
	//console.log('globalState',context.globalState.get('markdown-image-manage.hasBracket'));
	// if(!initPara()){return;} // 从配置中获取初始化参数
	let dispAnalyze = vscode.commands.registerCommand("markdown-image-manage.analyze", async (textEditor: vscode.TextEditor) => {
		if(!initPara()){return;} // 参数可能更新，重新从配置中获取初始化参数
		vscAnalyze();
	})
	let dispClean = vscode.commands.registerCommand("markdown-image-manage.clean", async () => {
		if(!initPara()){return;} // 参数可能更新，重新从配置中获取初始化参数
		vscClean();
	})
	let dispCleanLink = vscode.commands.registerCommand("markdown-image-manage.cleanlink", async () => {
		if(!initPara()){return;} // 参数可能更新，重新从配置中获取初始化参数
		vscClean(true);
	})
	let dispDownload = vscode.commands.registerCommand("markdown-image-manage.download", async () => {
		if(!initPara()){return;} // 参数可能更新，重新从配置中获取初始化参数
		vscDownload();
	})
	let dispUpload = vscode.commands.registerCommand("markdown-image-manage.upload", async () => {
		if(!initPara()){return;} // 参数可能更新，重新从配置中获取初始化参数
		vscUpload();
	})
	let dispUploadClip = vscode.commands.registerCommand("markdown-image-manage.uploadclip", async () => {
		if(!initPara()){return;} // 参数可能更新，重新从配置中获取初始化参数
		vscUpload(true);
	})
	let dispInsertClip = vscode.commands.registerCommand("markdown-image-manage.clipboardImage", async () => {
		if(!initPara()){return;} // 参数可能更新，重新从配置中获取初始化参数
		vscInsertClip();
	})
	let dispConvertImageFormat = vscode.commands.registerCommand("markdown-image-manage.convertImageFormat", async () => {
		if(!initPara()){return;} // 参数可能更新，重新从配置中获取初始化参数
		vscConvertImageFormat(); // encodeURI 
	})
	let dispMove = vscode.commands.registerCommand("markdown-image-manage.move", async () => {
		if(!initPara()){return;} // 参数可能更新，重新从配置中获取初始化参数
		vscMove();
	})
	context.subscriptions.push(dispAnalyze);
	context.subscriptions.push(dispClean);
	context.subscriptions.push(dispDownload);
	context.subscriptions.push(dispUpload);
	context.subscriptions.push(dispUploadClip);
	context.subscriptions.push(dispInsertClip);
	context.subscriptions.push(dispConvertImageFormat);
	context.subscriptions.push(dispMove);
}

// this method is called when your extension is deactivated
export function deactivate() {}
