const dayjs = require('dayjs');
const sqlite3 = require('sqlite3').verbose();
const dbName = 'records.db'
let dataSetting = {}
// 执行查询操作，返回记录集
function runQuery(db, sql, params) {
  return new Promise((resolve, reject) => {
    db.all(sql, params, (err, rows) => {
      if (err) {
        reject(err);
      } else {
        resolve(rows);
      }
    });
  });
}
// 不返回记录集
function runExec(db, sql, params) {
  return new Promise((resolve, reject) => {
    db.run(sql, params, function (err) {
      if (err) {
        reject(err);
      } else {
        resolve(this.changes);
      }
    });
  });
}
// 批量执行
function runBatchExec(db, sql) {
  return new Promise((resolve, reject) => {
    db.exec(sql, function (err) {  // 需要同时执行多条SQL
      if (err) {
        reject(err);
      }
      resolve(1)
    })
  });
}
// 创建一个连接到数据库的对象
// 创建表格（如果不存在）
async function insertData(records) {
  const db = new sqlite3.Database(dbName);
  let tick = records['tick']
  if (tick == null || tick == 0) {
    return 0
  }
  // 先检查是否已经有此tick数据，如果有则删除
  let changes = await runExec(db, 'delete FROM events where tick = ? ', [tick])
  console.log('删除tick数据条数: ', changes)
  // 删除完毕后才能插入，否则数据丢失
  // 将record 中的所有键值 插入数据
  let arr = []
  for (let k in records) {
    if (k != 'tick') {
      arr.push({
        name: k,
        count: records[k],
        tick
      })
    }
  }
  // 构建插入语句
  const placeholders = arr.map(() => '(?, ? , ? , ?)').join(', ');
  let tickDate = dayjs(new Date(tick)).format('YYYY-MM-DD')  // 需要用记录中的时间作为日期
  const values = arr.reduce((acc, curr) => acc.concat([curr.name, curr.count, curr.tick, tickDate]), []);
  // 执行一次性插入
  changes = await runExec(db, `INSERT INTO events (keyname, keycount , tick , date) VALUES ${placeholders}`, values)
  console.log(`A total of ${changes} rows have been inserted`);
  // 检查 events 是否有小于当天的数据,如果有，则需要触发转移
  setTimeout(doCleanData, 3000); // 过3秒后再处理，因为上面一行代码可能还在执行
  // 关闭数据库连接
  db.close();
  return changes
}
// 插入 分钟数据
async function insertMiniute(MinuteRecords) {
  // 202311232259 变成  2023-11-23
  function convert2Date(strTime) {
    return `${strTime.substring(0, 4)}-${strTime.substring(4, 6)}-${strTime.substring(6, 8)}`
  }
  const db = new sqlite3.Database(dbName);
  // 构建插入语句,同时插入 按键，
  const placeholders = MinuteRecords.map(() => '(?,?,?,?,?,?)').join(',');
  const values = MinuteRecords.reduce((acc, curr) => acc.concat([curr.Minute, curr.KeyCount, curr.MouseCount, curr.Distance, 0, convert2Date(curr.Minute)]), []);
  // 需要将 MinuteRecords 中 apps 属性信息进行设置和处理
  let appsPara = [], arrHolder = []
  MinuteRecords.forEach(x => {
    let len = Object.keys(x.Apps ?? {})
    if (len.length > 0) {
      let d = convert2Date(x.Minute)
      for (let key in x.Apps) {
        if (key == "") continue; // 为空值时跳过
        let app = x.Apps[key]
        appsPara.push(x.Minute, key, app.Key, app.Mouse, 0, d) // keyTime, appPath,keyCount,mouseCount freqType,date
        arrHolder.push(1)
      }
    }
  })
  // console.log("appsPara",appsPara)
  /* '202312010801',
'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
1,
1,
0,
'2023-12-01',
'202312010801',
'C:\\Windows\\System32\\cmd.exe',
6,
4,
0,*/
  const appsParaHolders = arrHolder.map(() => '(?,?,?,?,?,?)').join(',');
  // 执行一次性插入
  let changes = await runExec(db, `INSERT INTO statFreq (keyTime, keyCount , mouseCount, distance, freqType,date ) VALUES ${placeholders}`, values)
  console.log(`insertMiniute ${changes} rows have been inserted`);
  if (appsPara.length > 0) {
    changes = await runExec(db, `INSERT INTO appFreq (keyTime, appPath,keyCount,mouseCount,freqType,date ) VALUES ${appsParaHolders}`, appsPara)
    console.log(`inser appFreq ${changes} rows have been inserted`);
  }
  db.close();
}

async function getRecords(begin, end) {
  const db = new sqlite3.Database(dbName);
  let strNow = dayjs(new Date()).format('YYYY-MM-DD')
  // 查询记录集
  let arr = [];
  if (begin > end) {
    return arr;
  }
  let sql = ''
  if (begin == strNow && end == strNow) {
    sql = 'SELECT keyname, keycount, date, tick FROM events where date between ? and ?'
  } else {
    if (begin == end) {
      sql = 'SELECT keyname, keycount, date FROM stat where date between ? and ? '
    } else {
      sql = 'SELECT keyname, sum(keycount) as keycount, min(date) as date FROM stat where date between ? and ? group by keyname '
    }
    // sql = 'SELECT keyname, keycount, date FROM stat where date between ? and ? '
  }
  let rows = await runQuery(db, sql, [begin, end])
  // 输出记录集
  rows.forEach(function (row) {
    let tick = ''
    if (row.tick != null) {
      tick = row.tick
    }
    let keyname = row.keyname, keycount = row.keycount, date = row.date;
    arr.push({ keyname, keycount, date, tick });
  });
  // 关闭数据库连接
  db.close();
  return arr
}
// 将 events的旧数据统计后转移到stat 表中
async function doCleanData() {
  const db = new sqlite3.Database(dbName);
  console.log('doCleanData')
  let strNow = dayjs(new Date()).format('YYYY-MM-DD')
  // 执行sql处理，插入统计数据
  try {
    let rows = await runQuery(db, 'SELECT tick FROM events where date<? limit 1', [strNow])
    // 输出记录集
    if (rows.length > 0) {
      let lines = await runExec(db, `INSERT INTO stat (keyname, keycount,date) SELECT keyname, sum(keycount), date FROM events where date < ? group by date,keyname`, [strNow])
      console.log('转移数据条数: ', lines)
      // 删除 events 中的旧数据
      if (lines > 0) {
        await runExec(db, 'delete FROM events where date < ? ', [strNow])
      }
    }
    // 如果 statFreq 表中的日期有小于指定日期的，需要 
    let before = dataSetting.minuteKeepDays ?? 7
    let beforeDays = dayjs().subtract(before, 'day').format('YYYY-MM-DD'); // 如果 statFreq中有小于 beforeDays 日期的数据则要执行删除动作
    let lines = await runExec(db, `delete FROM statFreq where freqType = 0 and date < ? `, [beforeDays])
    console.log('删除分钟统计条数: ', lines)
    // 删除 events 中的旧数据
    lines = await runExec(db, `delete FROM appFreq where freqType = 0 and date < ? `, [beforeDays])
    console.log('删除分钟应用条数: ', lines)
    // statFreq 的 小时日期数据小于昨天 则要进行24小时数据整理函数
    rows = await runQuery(db, "SELECT COALESCE(max(date),'1900-00-00') as maxDate FROM statFreq where freqType = 1", [])
    let maxDate = rows[0].maxDate;
    let yesterday = dayjs().subtract(1, 'day').format('YYYY-MM-DD')
    console.log(before, beforeDays, maxDate, yesterday)
    if (maxDate < yesterday) {
      // 需要将  yesterday 天的数据按分钟统计，并插入到表中
      console.log('create hours')
      lines = await runExec(db, `INSERT INTO statFreq (keyTime, keyCount,mouseCount,distance,freqType,date) 
      SELECT SUBSTR(keyTime,0,11) as kt ,sum(keyCount),sum(mouseCount),sum(distance),1,date FROM statFreq 
      where freqType = 0 and date between ? and ? group by kt
      `, [maxDate, yesterday])
      console.log('插入小时统计条数: ', lines)
      lines = await runExec(db, `INSERT INTO appFreq (keyTime,appPath, keyCount,mouseCount,freqType,date) 
      SELECT SUBSTR(keyTime,0,11) as kt, appPath, sum(keyCount),sum(mouseCount),count(keyTime),date FROM appFreq 
      where freqType = 0 and date between ? and ? group by appPath, kt
      `, [maxDate, yesterday])
      console.log('插入小时应用条数: ', lines)
    }
  } catch (err) {
    console.error(err);
  }
  db.close();
}
/*
(async ()=>{
  arr = await getRecords()
  console.log(arr)
})()
*/
// 数据库中的相关配置信息，主要用于统计
let globalTopN = 10;
let globalAppTopN = 10;
async function getDataSetting() {
  const db = new sqlite3.Database(dbName);
  // 查询记录集
  let sql = `SELECT keyname,val from dataSetting2 
    union
    select 'mapDetail',mapDetail from keymaps join dataSetting2 on mapName = val where keyname='keymap' `
  let rows = await runQuery(db, sql, [])
  // 输出记录集
  dataSetting = {}
  rows.forEach(r => {
    dataSetting[r.keyname] = r.val
  })
  return dataSetting
  // 关闭数据库连接
}
// 获取全部键盘
async function getKeymaps() {
  const db = new sqlite3.Database(dbName);
  // 查询记录集
  let arr = [];
  let sql = 'SELECT mapName,mapDetail FROM keymaps'
  let rows = await runQuery(db, sql, [])
  // 输出记录集
  rows.forEach(function (row) {
    let mapName = row.mapName, mapDetail = row.mapDetail;
    arr.push({ mapName, mapDetail });
  });

  // 关闭数据库连接
  db.close();
  return arr
}
// 保存统计的相关配置
async function setDataSetting(hash) {
  const db = new sqlite3.Database(dbName);
  // 动态保存参数  mapDetail 参数在另外的表中，无需保存
  delete hash['mapDetail'];
  const placeholders = Object.keys(hash).map(() => '(?, ?)').join(', ');
  // 需要将内容全部按数组顺序排列
  const values = Object.entries(hash).flat(3); // Infinity 数组展开配置参数表
  await runExec(db, `INSERT OR REPLACE INTO dataSetting2 (keyname, val) VALUES ${placeholders}`, values)
  // 输出记录集
  globalTopN = hash.topN
  globalAppTopN = hash.appTopN
  // 关闭数据库连接
  db.close();
}
// 操作 keymap , 0=删除 1=新增 2=更新
async function optKeyMap(data) {
  console.log('optKeyMap:', data.mapName, data.flag)
  const db = new sqlite3.Database(dbName);
  // 查询记录集
  let { mapName, mapDetail, flag } = data;
  if (mapName.toLowerCase() == 'default') {
    return 0
  }
  let sql = '', para = []
  switch (flag) {
    case 0:
      sql = 'delete from keymaps where mapName = ?'
      para = [mapName]
      break;
    case 1:
      sql = 'insert into keymaps(mapName, mapDetail) values(?,?) '
      para = [mapName, mapDetail]
      break;
    case 2:
      sql = 'update keymaps set mapDetail = ? where mapName = ?'
      para = [mapDetail, mapName]
      break;
  }
  await runQuery(db, sql, para)
  // 关闭数据库连接
  db.close();
  return 1
}
// 获取全部历史天数
async function getHistoryDate() {
  const db = new sqlite3.Database(dbName);
  // 查询记录集
  let arr = [];
  let sql = 'SELECT date FROM stat group by date order by date desc'
  let rows = await runQuery(db, sql, [])
  // 输出记录集
  rows.forEach(function (row) {
    arr.push(row.date);
  });
  // 关闭数据库连接
  db.close();
  return arr
}

//获取各类统计信息到一个hash中
async function statData(begin, end) {
  const db = new sqlite3.Database(dbName);
  let pro1 = new Promise((resolve, reject) => {
    // 查询鼠标移动距离
    let sql = "select keycount,date from stat where date between ? and ? and keyname = 'mouseDistance' order by date"
    db.all(sql, [begin, end], function (err, rows) {
      if (err) {
        reject(err);
      }
      resolve(rows)
    });
  })
  // 查询鼠标点击数量和按键数量总和
  let pro2 = new Promise((resolve, reject) => {
    // 查询记录集
    let sql = `select sum(keycount) as keycount,date,'mouse' as keyname from stat 
    where date between ? and ? and keyname in ('LButton','RButton','MButton','WheelDown','WheelUp') group by date
UNION
select sum(keycount) as keycount,date,'keyboard' from stat 
where date between ? and ? and keyname not in ('mouseDistance','LButton','RButton','MButton','WheelDown','WheelUp')
and keyname not like('App-%')
group by date`
    db.all(sql, [begin, end, begin, end], function (err, rows) {
      if (err) {
        reject(err);
      }
      resolve(rows);
    });
  })
  // 查询统计范围内按键数 top10
  let pro3 = new Promise((resolve, reject) => {
    // 查询记录集
    let sql = `
WITH
temp AS (
  SELECT * FROM "stat"  where date between ? and ? )
select keycount,date,keyname from temp where keyname in
(select keyname from temp where keyname not in ('mouseDistance','LButton','RButton','MButton','WheelDown','WheelUp')
 and keyname not like('App-%')
group by keyname order by sum(keycount) desc limit ${globalTopN}
) order by date`
    db.all(sql, [begin, end], function (err, rows) {
      if (err) {
        reject(err);
      }
      resolve(rows)
    });
  })
  // 查询统计范围内APP操作数的前 topN
  let pro4 = new Promise((resolve, reject) => {
    // 查询记录集
    let sql = `WITH
temp AS (
  SELECT * FROM "stat"  where keyname like 'APP-%' and date between ? and ?),
	temp2 as (
  SELECT  REPLACE(REPLACE(keyname,'App-Mouse-',''),'App-Key-','') as pathname,sum(keycount) as count FROM temp  
    group by pathname order by count desc limit ${globalAppTopN}
)
select * from temp where keyname in ( select 'App-Mouse-'||pathname from temp2 ) 
union
select * from temp where keyname in ( select 'App-Key-'||pathname from temp2 ) 
    `
    db.all(sql, [begin, end], function (err, rows) {
      if (err) {
        reject(err);
      }
      resolve(rows)
    });
  })
  let arr = await Promise.all([pro1, pro2, pro3, pro4]);
  db.close();
  return arr;
}
// 删除 events 或 stat 数据
async function deleteData(date, flag) {
  const db = new sqlite3.Database(dbName);
  return new Promise((resolve, reject) => {
    // 查询记录集 
    let sql = ''
    //const placeholders = date.map(() => '?').join(',');
    const placeholders = date.map((x) => "'" + x + "'").join(',');
    if (flag == 0) {
      sql = 'delete FROM events where tick in(' + placeholders + ') '
    } else if (flag == 1) {
      sql = `delete FROM stat where date in(${placeholders}) ; 
      delete FROM statFreq where date in(${placeholders}) ;
      delete FROM appFreq where date in(${placeholders}) ;
      `  // 需要支持多个SQL
      // date = date.concat(date, date);  // 有3个 placeholders 重复3次
    }
    console.log(sql, date)
    if (sql != '') {
      db.exec(sql, function (err) {
        if (err) {
          reject(err);
        }
        resolve(1)
      });
    }
    // 关闭数据库连接
    db.close();
  })
}

// 更新数据库表结构
async function updateDBStruct() {
  // dataSetting  ->  dataSetting2 更新表结构
  const db = new sqlite3.Database(dbName);
  // 查询记录集
  let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name='dataSetting2';"
  let rows = await runQuery(db, sql, [])
  // 输出记录集
  if (rows.length == 0) {
    console.log('Need updateDBStruct')
    // 需要创建表 dataSetting2 数据从 dataSetting 中过来
    sql = `CREATE TABLE dataSetting2(
	  keyname TEXT,
	  val TEXT,
	   PRIMARY KEY (keyname)
	);
	insert into dataSetting2 
	  select 'keymap' as keyname ,  keymap as  val  from dataSetting
	  union 
	  select 'screenSize' as keyname , screenSize as  val  from dataSetting
	  union 
	  select 'mouseDPI' as keyname , mouseDPI as  val  from dataSetting
	  union 
	  select 'topN' as keyname , topN as  val  from dataSetting;`
    await runBatchExec(db, sql);
  }
  // 查询记录集
  sql = "SELECT name FROM sqlite_master WHERE type='table' AND name='statFreq';"
  rows = await runQuery(db, sql, [])
  // 输出记录集
  if (rows.length == 0) {
    sql = `CREATE TABLE statFreq (
          keyTime TEXT, 
          keyCount INTEGER, 
          mouseCount INTEGER, 
          distance INTEGER,  
          freqType INTEGER,
          date TEXT
        );
        CREATE INDEX statFreq_date_IDX ON statFreq (date);
        CREATE INDEX statFreq_type_IDX ON statFreq (freqType);
        CREATE TABLE appFreq (
          keyTime TEXT, 
          appPath TEXT, 
          keyCount INTEGER, 
          mouseCount INTEGER, 
          freqType INTEGER,
          date TEXT
        );
        CREATE INDEX appFreq_date_IDX ON appFreq (date);
        CREATE INDEX appFreq_type_IDX ON appFreq (freqType);
        `
    await runBatchExec(db, sql);
  }
  // 关闭数据库连接
  db.close();
}

// 获取最新的分钟信息
async function getLastMinute() {
  const db = new sqlite3.Database(dbName);
  // 查询记录集
  let lastObj = {};
  let sql = 'SELECT keyTime,keyCount,mouseCount,distance FROM statFreq where freqType = 0 order by date desc, keyTime desc limit 1'
  let rows = await runQuery(db, sql, [])
  rows.forEach(function (row) {
    lastObj = { "Distance": row.distance, "KeyCount": row.keyCount, "Minute": row.keyTime, "MouseCount": row.mouseCount };
  });
  db.close();
  return lastObj
}

// 获取分钟数据
async function getMinuteRecords(beginDate, endDate, freqType, isApp) {
  const db = new sqlite3.Database(dbName);
  // 查询记录集
  freqType = parseInt(freqType, 10)
  let sqlFreqType = `freqType = ${freqType} ` // 分钟数据
  let arr = []
  let sql = 'SELECT keyTime,keyCount,mouseCount,distance,date FROM statFreq where ' + sqlFreqType + ' and date between ? and ? order by date'
  if (isApp) {
    if (freqType > 0) {
      sqlFreqType = 'freqType > 0'  // 此时表示为分钟数
    }
    sql = 'SELECT keyTime, keyCount , mouseCount,appPath,date,freqType FROM appFreq where ' + sqlFreqType + ' and date between ? and ? order by date'
  }
  let rows = await runQuery(db, sql, [beginDate ?? '', endDate ?? ''])
  rows.forEach((row) => {
    if (isApp) {
      arr.push({ "Apps": row.appPath, "KeyCount": row.keyCount, "Minute": row.keyTime, "MouseCount": row.mouseCount, "Date": row.date, "Duration": row.freqType });
    }
    else {
      arr.push({ "Distance": row.distance, "KeyCount": row.keyCount, "Minute": row.keyTime, "MouseCount": row.mouseCount, "Date": row.date });
    }
  });
  db.close();
  return arr
}
module.exports = {
  insertData, getRecords, getDataSetting, setDataSetting, getKeymaps, optKeyMap, getHistoryDate,
  statData, deleteData, dbName, updateDBStruct, insertMiniute, getLastMinute, getMinuteRecords
};