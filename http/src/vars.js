const path = require('path');
const dbName = 'records.db'
const basePath = path.join(__dirname,'../../')
const iniPath = path.join(basePath, 'showKeyBoard.ini')
const keyPath = path.join(basePath, 'keyList.txt')
const backupPath = path.join(basePath, 'backup/')
const dbsPath = path.join(basePath, 'dbs/')
const defaultIniPath = path.join(__dirname, './showKeyBoard.desc.ini')
const updateTimePath = path.join(__dirname, 'updateTime.txt');
const lastRecordPath = path.join(__dirname, 'lastRecord.json');
const pidfilePath = path.join(__dirname, 'kbserver.pid');

module.exports = {
  dbName, basePath, iniPath, keyPath, backupPath, dbsPath, defaultIniPath, updateTimePath, lastRecordPath, pidfilePath
  };