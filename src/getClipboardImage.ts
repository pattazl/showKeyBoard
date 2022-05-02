import * as path from 'path'
import { spawn } from 'child_process'
import * as os from 'os'
import * as fs from 'fs-extra'
import * as isWsl from 'is-wsl'
import { logger } from './common'
import { getLang } from './lang'
// import macClipboardScript from './clipboard/mac.applescript' //'./clipboard/mac.applescript'
// import windowsClipboardScript from './clipboard/windows.ps1'
// import windows10ClipboardScript from './clipboard/windows10.ps1'
// import linuxClipboardScript from './clipboard/linux.sh'
// import wslClipboardScript from './clipboard/wsl.sh'

export type Platform = 'darwin' | 'win32' | 'win10'| 'linux' | 'wsl'

// export var cbEvent = new EventEmitter(); 

const getCurrentPlatform = (): Platform => {
  const platform = process.platform
  if (isWsl) {
    return 'wsl'
  }
  if (platform === 'win32') {
    const currentOS = os.release().split('.')[0]
    if (currentOS === '10') {
      return 'win10'
    } else {
      return 'win32'
    }
  } else if (platform === 'darwin') {
    return 'darwin'
  } else {
    return 'linux'
  }
}

// const platform2ScriptContent: {
//   [key in Platform]: string
// } = {
//   darwin: macClipboardScript,
//   win32: windowsClipboardScript,
//   win10: windows10ClipboardScript,
//   linux: linuxClipboardScript,
//   wsl: wslClipboardScript
// }
/**
 * powershell will report error if file does not have a '.ps1' extension,
 * so we should keep the extension name consistent with corresponding shell
 */
const platform2ScriptFilename: {
  [key in Platform]: string
} = {
  darwin: 'mac.applescript',
  win32: 'windows.ps1',
  win10: 'windows10.ps1',
  //win: 'win.ps1',
  linux: 'linux.sh',
  wsl: 'wsl.sh'
}

export const getClipboardImage = async (imagePath: string): Promise<string> => {
  let runPath = path.resolve(__dirname+'/clipboard');
  return await new Promise<string>((resolve: Function, reject: Function): void => {
    const platform = getCurrentPlatform()
    const scriptPath = path.join(runPath, platform2ScriptFilename[platform])
    // If the script does not exist yet, we need to write the content to the script file
    // 需要将相关脚本复制过去
    if (!fs.existsSync(scriptPath)) {
      return reject(new Error(getLang('notfoundscript',scriptPath)))
    }
    let execution
    if (platform === 'darwin') {
      execution = spawn('osascript', [scriptPath, imagePath])
    } else if (platform === 'win10' || platform === 'win32' ) { 
      execution = spawn('powershell', [
        '-noprofile',
        '-noninteractive',
        '-nologo',
        '-sta',
        '-executionpolicy', 'unrestricted',
        // '-windowstyle','hidden',
        // '-noexit',
        '-file', scriptPath,
        imagePath
      ])
    } else {
      execution = spawn('sh', [scriptPath, imagePath])
    }

    execution.stderr.on('data', function (data) {
      logger.error(data.toString(),false);
    });
    execution.stdout.on('data', (data: Buffer) => {
      if (platform === 'linux') {
        if (data.toString().trim() === 'no xclip or wl-clipboard') {
          return reject(new Error('Please install xclip(for x11) or wl-clipboard(for wayland) before paste image'))
        }
      }
      const imgPath = data.toString().trim()
      // if the filePath is the real file in system
      // we should keep it instead of removing
      // let shouldKeep = false
      // // in macOS if your copy the file in system, it's basename will not equal to our default basename
      // if (path.basename(imgPath) !== path.basename(imagePath)) {
      //   // if the path is not generate
      //   // but the path exists, we should keep it
      //   if (fs.existsSync(imgPath)) {
      //     shouldKeep = true
      //   }
      // }
      // if the imgPath is invalid
      if (imgPath !== 'no image' && !fs.existsSync(imgPath)) {
        return reject(new Error(`Can't find ${imgPath}`))
      }
      resolve(imgPath)
    })
  })
}

