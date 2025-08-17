/////////////////////////////////
const fs = require('fs');
const fsPro = require('fs').promises;
const iconv = require('iconv-lite');
const path = require('path');
const WebSocket = require('ws');
const http = require('http');
const express = require('express')
const { insertData, getDataSetting, setDataSetting, getKeymaps, optKeyMap, deleteData, dbName,
    updateDBStruct, insertMiniute, getLastMinute, cleanErrStat } = require('./records');
const dayjs = require('dayjs');
const net = require('net');
const app = express()
const ini = require('ini')
const fontList = require('font-list')
const JSZip = require("jszip");
const os = require('os');
// 2个配置文件
const basePath = path.join(__dirname, '../../')
const iniPath = path.join(basePath, 'showKeyBoard.ini')
const keyPath = path.join(basePath, 'keyList.txt')
const backupPath = path.join(basePath, 'backup/')
const dbsPath = path.join(basePath, 'dbs/')
const defaultIniPath = path.join(__dirname, 'showKeyBoard.desc.ini')
const updateTimePath = path.join(__dirname, 'updateTime.txt');
const lastRecordPath = path.join(__dirname, 'lastRecord.json');
const pidfilePath = path.join(__dirname, 'kbserver.pid');


//////

var config = {} // 配置文件
config = getConfig()

// 根据路径, ANSI->UTF8 读取
function iniAnsiRead(file) {
    const buffer = fs.readFileSync(file);
    content = iconv.decode(buffer, 'gbk');
    return content
}
// 完整的获取配置文件
function getConfig() {
    let config = {}
    let defaultConfig = {}
    if (fs.existsSync(iniPath)) {
        config = ini.parse(iniAnsiRead(iniPath))
    }
    if (fs.existsSync(defaultIniPath)) {
        defaultConfig = ini.parse(fs.readFileSync(defaultIniPath, 'utf-8'))
    }
    //console.log(defaultConfig)
    mergeObjects(config, defaultConfig)
    // 配置文件修改
    if ((config.common?.shareDbName || '') == '') {
        config.common.shareDbName = os.hostname()
    }
    return config
}
// default 中存在的hash值复制到 obj 中
function mergeObjects(obj1, obj2) {
    // 遍历obj2的属性
    for (let prop in obj2) {
        // 如果obj1对应的属性不存在，则直接将obj2的属性添加到obj1中
        if (typeof obj1[prop] === 'undefined') {
            obj1[prop] = obj2[prop];
        } else if (typeof obj1[prop] === 'object' && typeof obj2[prop] === 'object') {
            // 如果两个属性都是普通对象，则递归调用mergeObjects函数
            mergeObjects(obj1[prop], obj2[prop]);
        }
    }
    return obj1;
}
////////////////////////////////////////////////////
// 生成相关 zip文件，处理方式根据函数
function zipCore(cbFun, isShare /**是否共享同步 */) {
    const zip = new JSZip();
    let fileContent;
    if (isShare) {
        let newDbName = `${config.common.shareDbName}_${parseInt(new Date().getTime() / 1000)}.db`  // 秒为单位的时间戳
        fileContent = fs.readFileSync(dbName);
        zip.file(newDbName, fileContent);
    } else {
        fileContent = fs.readFileSync(iniPath);
        zip.file(path.basename(iniPath), fileContent);
        fileContent = fs.readFileSync(keyPath);
        zip.file(path.basename(keyPath), fileContent);
        fileContent = fs.readFileSync(dbName);
        zip.file(dbName, fileContent);
    }
    zip.generateAsync({
        type: 'nodebuffer',
        compression: 'DEFLATE',
        compressionOptions: {
            level: 5  /** 压缩比 0-9 低到高 */
        }
    }).then(function (content) {
        // see FileSaver.js
        cbFun(content)
    });
}

function unZipCore(hash, fun) {
    let content = fs.readFileSync(hash.path)
    return new Promise((resolve, reject) => {
        const zip = new JSZip();
        zip.loadAsync(content)
            .then(async function (contents) {
                //console.log( JSON.stringify(contents))
                for (let relativePath in contents.files) {
                    let file = contents.files[relativePath]
                    //console.log(relativePath,file.dir,file._data.compressedContent)
                    if (!file.dir) { // 过滤掉目录
                        const fileData = await file.async("uint8array"); // 以uint8array格式读取文件内容
                        // 在此处执行对解压出的文件的操作，例如保存到本地
                        let configPath = fun(relativePath)
                        fs.writeFileSync(configPath, fileData)
                        hash.msg.push(relativePath + ' Ok')
                    }
                }
                /*
                contents.forEach(async function (relativePath, file) {
                  console.log(relativePath)
                  if (!file.dir) { // 过滤掉目录
                    const fileData = await file.async("uint8array"); // 以uint8array格式读取文件内容
                    // 在此处执行对解压出的文件的操作，例如保存到本地
                    let configPath = configHash[relativePath]
                    fs.writeFileSync(configPath, fileData)
                    hash.msg.push(relativePath + ' Ok\n')
                    console.log('OKOK')
                  }
                });*/
                resolve(0)
            })
            .catch(function (error) {
                let msg = "解压缩失败: "
                console.error(msg, error);
                hash.msg.push(msg)
                resolve(0)
            });
    })
}
///////////////////////////////////

// 启动定时生成共享文件 shareDbPath shareDbName shareDbHour
function autoShare() {
    let hour = parseFloat(config.common.shareDbHour)
    hour = isNaN(hour) ? 0 : hour  // 默认0小时，不做备份
    if (hour > 0) {
        autoShareCore()
        setInterval(autoShareCore, 60 * 60 * 1000 * hour); // 定时检查
    }
}
autoShare()
// 需要记录 已经解压的文件清单时间，如果变化才重新解压覆盖
async function autoShareCore() {
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
    let dbsPath = path.resolve(basePath, 'dbs/')
    try {
        await fsPro.mkdir(dbsPath, { recursive: true });
    } catch (err) {
        console.error(`目录创建失败: ${err.message}`);
        return
    }
    let shareFileName = path.resolve(shareFilePath, config.common.shareDbName + '.zip')
    // 将当前 records.db 按规范名称压缩
    zipCore(function (content) {
        // see FileSaver.js
        fs.writeFileSync(shareFileName, content);
    }, true)
    // 解压已有共享文件到目录 系统安装目录的 /dbs 下 dbsPath
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
        // 如果是新文件需要解药，旧文件的话，需要不是自己的文件，且修改时间更新
        console.log(sharedDbsInfo[fileId])
        let needUnzip = false
        if (sharedDbsInfo[fileId] == null) {
            needUnzip = true
        } else {
            let fileName = sharedDbsInfo[fileId].name
            let dbsCreateTime = new Date(parseInt(fileName.split('_').pop()) * 1000)
            console.log(dbsCreateTime)
            if (newTime > dbsCreateTime) {
                // 文件更新需要删除原先的db文件
                console.log('delete')
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



