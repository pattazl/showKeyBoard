// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';

// this method is called when your extension is activated
// your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
	
	// Use the console to output diagnostic information (console.log) and errors (console.error)
	// This line of code will only be executed once when your extension is activated
	console.log('Congratulations, your extension "markdown-image-manage" is now active!');

	// The command has been defined in the package.json file
	// Now provide the implementation of the command with registerCommand
	// The commandId parameter must match the command field in package.json
	/*let disposable0 = vscode.commands.registerCommand('markdown-image-manage.helloWorld', () => {
		// The code you place here will be executed every time your command is executed
		// Display a message box to the user
		vscode.window.showInformationMessage('Hello World2222 from markdown image manage!');
	});
	let disposable = vscode.commands.registerCommand("markdown-image-manage.helloWorld", async () => {
		let answer = await vscode.window.showInformationMessage("How was your day ?", "good", "bad",)
		if (answer === "bad") {
			vscode.window.showErrorMessage("sorry to hear it", "1", "2","3","4")
		} else {
			console.log({ answer })
			vscode.window.showWarningMessage("sorry to hear it")
		}
	})*/
	let dispAnalyze = vscode.commands.registerCommand("markdown-image-manage.analyze", async () => {
		await vscode.window.showInformationMessage("analyze ?")
	})
	let dispClean = vscode.commands.registerCommand("markdown-image-manage.clean", async () => {
		await vscode.window.showInformationMessage("clean ?")
	})
	let dispDownload = vscode.commands.registerCommand("markdown-image-manage.download", async () => {
		await vscode.window.showInformationMessage("download ?")
	})
	let dispUpload = vscode.commands.registerCommand("markdown-image-manage.upload", async () => {
		await vscode.window.showInformationMessage("upload ?")
	})
	let dispMove = vscode.commands.registerCommand("markdown-image-manage.move", async () => {
		await vscode.window.showInformationMessage("move ?")
	})
	context.subscriptions.push(dispAnalyze);
	context.subscriptions.push(dispClean);
	context.subscriptions.push(dispDownload);
	context.subscriptions.push(dispUpload);
	context.subscriptions.push(dispMove);
}

// this method is called when your extension is deactivated
export function deactivate() {}
