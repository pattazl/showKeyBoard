/////////////////////////////////
const fs = require('fs');
const fsPro = require('fs').promises;
const path = require('path');
const { exec } = require('child_process');
// 2个配置文件
const {basePath} =require('./vars')
const {config,zipCore,unZipCore} =require('./common')


////////////////////////////////////////////////////
// 启动定时生成共享文件 shareDbPath shareDbName shareDbHour shareDbExec
function autoShare() {
    let hour = config.common.shareDbHour
    if (hour > 0) {
        autoShareCore()
        setInterval(autoShareCore, 60000); // 每分钟定时检查
    }
}
// 需要记录 已经解压的文件清单时间，如果变化才重新解压覆盖
async function autoShareCore() {
    // 正式开始
    let shareFilePath = path.resolve(basePath, config.common.shareDbPath)
    // 递归创建目录
    try {
        // 尝试创建目录，recursive: true 表示递归创建不存在的父目录
        await fsPro.mkdir(shareFilePath, { recursive: true });
        // console.log(`目录创建成功: ${shareFilePath}`);
    } catch (err) {
        console.error(`目录创建失败: ${err.message}`);
        //throw err; // 抛出错误以便调用者处理
        return
    }
    let shareFileName = path.resolve(shareFilePath, config.common.shareDbName + '.zip')
    // 需要判断当前压缩文件是否存在，如果存在则需要判断日期是否需要重新压缩替换
    let needShare = false;
    if (fs.existsSync(shareFileName)) {
        try {
            const stats = fs.statSync(shareFileName);
            const mtime = stats.mtime;
            let diff = new Date().getTime() - mtime.getTime()
            // console.log(shareFileName, mtime.getTime(), diff) 超过时间才备份
            if (diff > config.common.shareDbHour * 3600 * 1000) {
                needShare = true // 文件旧，需要备份文件
            }
        } catch (err) {
            console.error('压缩当前数据失败:', err);
        }
    }else{
        needShare = true // 不存在，需要备份新文件
    }
    // 需要生成备份文件
    if (needShare) {
        // 将当前 records.db 按规范名称压缩
        zipCore(function (content) {
            // see FileSaver.js
            fs.writeFileSync(shareFileName, content);
            // 可以执行相关的命令，用于触发自动同步
            let cmdLine = config.common.shareDbExec?.trim()
            if(cmdLine!=''){
                doExec(cmdLine)
            }
        }, true)
    }
    // 解压共享文件
    await unzipShared(shareFilePath)
}

async function unzipShared(shareFilePath) {
    // 解压已有共享文件到目录 系统安装目录的 /dbs 下 dbsPath
    let dbsPath = path.resolve(basePath, 'dbs/')
    try {
        await fsPro.mkdir(dbsPath, { recursive: true });
    } catch (err) {
        console.error(`目录创建失败: ${err.message}`);
        return
    }
    // 读取 dbsPath 的全部文件列表
    let zipInfo = await getFilesInfo(shareFilePath, '.zip', (x) => x.split('.').slice(0, -1).join('.')) // 去掉最后的 .
    // 读取 dbs 目录下的全部文件列表，获得修改时间
    let sharedDbsInfo = await getFilesInfo(dbsPath, '.db', (x) => x.split('_').slice(0, -1).join('_')) //去掉最后的 _
    // console.log(JSON.stringify(zipInfo))
    // 循环判断是否需要解压.，需要排除自己的和文件没变化的
    for (let fileId in zipInfo) {
        // 需要不是自己的文件
        if (config.common.shareDbName == fileId) {
            continue;
        }
        let fileName = zipInfo[fileId].name
        let ctime = zipInfo[fileId].createTime
        let newTime = new Date(ctime.getTime() - 60 * 1000) // 文件创建时间大于1分钟
        // 如果是新文件需要解压，旧文件的话，需要不是自己的文件，且修改时间更新
        let needUnzip = false
        if (sharedDbsInfo[fileId] == null) {
            needUnzip = true
        } else {
            let fileName = sharedDbsInfo[fileId].name
            let dbsCreateTime = new Date(parseInt(fileName.split('_').pop()) * 1000)
            if (newTime > dbsCreateTime) {
                // 文件更新需要删除原先的db文件
                console.log('delete shared db')
                fs.unlinkSync(sharedDbsInfo[fileId].path)
                needUnzip = true
            }
        }
        if (needUnzip) {
            // 需要解压,同时解压的时候，在文件末尾增加时间戳,格式为 用户名_备份时间_解压时间
            let zipFile = path.resolve(shareFilePath, fileName)
            console.log('unzip:', zipFile)
            let hash = { path: zipFile, msg: [] }
            await unZipCore(hash, (filename) => path.resolve(dbsPath, filename))
        }
    }
}

/**
 * 读取指定目录下所有.zip文件的信息
 * @param {string} directory - 要扫描的目录路径
 * @returns {Promise<Object>} 包含所有zip文件信息的哈希对象
 */
async function getFilesInfo(directory, fileType, keyFun) {
    const filesInfo = {};
    if (!fs.existsSync(directory)) {
        return filesInfo
    }
    try {
        // 读取目录中的所有文件
        const files = await fsPro.readdir(directory, { withFileTypes: true });
        // 存储结果的哈希对象
        for (const file of files) {
            // 检查是否为文件且以.zip结尾
            if (file.isFile() && path.extname(file.name).toLowerCase() === fileType) {
                const filePath = path.join(directory, file.name);
                // 获取文件的详细信息
                const stats = await fsPro.stat(filePath);
                let keyName = keyFun(file.name)
                // 将信息存入哈希对象，键为文件名，值为创建时间
                filesInfo[keyName] = {
                    name: file.name,
                    path: filePath,
                    createTime: stats.mtime,  // 修改时间 stats.birthtime, // 创建时间
                    // createTimeISO: stats.birthtime.toISOString() // ISO格式的创建时间
                };
            }
        }
        return filesInfo;
    } catch (err) {
        console.error('读取目录时发生错误:', err);
        throw err;
    }
}

// 执行额外命令
function doExec(command)
{
  // 开启子进程执行命令
  console.log(`${new Date()}-exec command: ${command}`);
  const child = exec(command, { 
    cwd: basePath
  }, (error, stdout, stderr) => {
    if (error) {
      console.error(`exec error: ${error.message}`);
      return;
    }
    if (stderr) {
      console.error(`error output: ${stderr}`);
      return;
    }
    console.log(`output:\n${stdout}`);
  });
  // 可以监听子进程的退出事件
  child.on('exit', (code) => {
    console.log(`exec exit code: ${code}`);
  });
  return child;
}
// 启动后自动触发
// autoShare()
module.exports = { autoShare };
