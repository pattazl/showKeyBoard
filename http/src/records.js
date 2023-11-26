const dayjs = require('dayjs');
const sqlite3 = require('sqlite3').verbose();
const dbName = 'records.db'
// 创建一个连接到数据库的对象
// 创建表格（如果不存在）
function insertData(records) {
  return new Promise((resolve, reject) => {
    const db = new sqlite3.Database(dbName);
    let tick = records['tick']
    if (tick == null || tick == 0) {
      resolve(0)
      return
    }
    // 先检查是否已经有此tick数据，如果有则删除
    db.run('delete FROM events where tick = ? ', [tick], function (err) {
      if (err) {
        return console.error(err.message);
      }
      console.log('删除tick数据条数: ', this.changes)
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
      db.run(`INSERT INTO events (keyname, keycount , tick , date) VALUES ${placeholders}`, values, function (err) {
        if (err) {
          console.error(err.message);
          reject(err)
        }
        console.log(`A total of ${this.changes} rows have been inserted`);
        resolve(this.changes)
      });
    });
    // 检查 events 是否有小于当天的数据,如果有，则需要触发转移
    setTimeout(doCleanData, 3000); // 过3秒后再处理，因为上面一行代码可能还在执行
    // 关闭数据库连接
    db.close();
  })
}
// 插入 分钟数据
function insertMiniute(MinuteRecords) {
  // 202311232259 变成  2023-11-23
  function convert2Date( strTime ){
    return `${str.substring(0, 4)}-${str.substring(4, 6)}-${str.substring(6, 8)}`
  }
  return new Promise((resolve, reject) => {
    const db = new sqlite3.Database(dbName);
    // 构建插入语句,同时插入 按键，
    const placeholders = MinuteRecords.map(() => '(?,?,?,?,?,?)').join(',');
    const values = MinuteRecords.reduce((acc, curr) => acc.concat([curr.Minute, curr.KeyCount, curr.MouseCount, curr.Distance, 0,convert2Date(curr.Minute)]), []);
    // 执行一次性插入
    db.run(`INSERT INTO statFreq (keyTime, keyCount , mouseCount, distance, freqType,date ) VALUES ${placeholders}`, values, function (err) {
      if (err) {
        console.error(err.message);
        reject(err)
      }
      console.log(`insertMiniute ${this.changes} rows have been inserted`);
      resolve(this.changes)
    });
    db.close();
  })
}

function getRecords(begin, end) {
  const db = new sqlite3.Database(dbName);
  let strNow = dayjs(new Date()).format('YYYY-MM-DD')
  return new Promise((resolve, reject) => {
    // 查询记录集
    let arr = [];
    if (begin > end) {
      resolve(arr)
      return;
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
    db.all(sql, [begin, end], function (err, rows) {
      if (err) {
        reject(err);
      }
      // 输出记录集
      rows.forEach(function (row) {
        let tick = ''
        if (row.tick != null) {
          tick = row.tick
        }
        let keyname = row.keyname, keycount = row.keycount, date = row.date;
        arr.push({ keyname, keycount, date, tick });
      });
      resolve(arr)
    });
    // 关闭数据库连接
    db.close();
  })
}
// 将 events的旧数据统计后转移到stat 表中
function doCleanData() {
  const db = new sqlite3.Database(dbName);
  console.log('doCleanData')
  let strNow = dayjs(new Date()).format('YYYY-MM-DD')
  // 执行sql处理，插入统计数据
  db.all('SELECT tick FROM events where date<? limit 1', [strNow], function (err, rows) {
    if (err) {
      //reject(err);
      return console.error(err.message);
    }
    // 输出记录集
    if (rows.length > 0) {
      db.run(`INSERT INTO stat (keyname, keycount,date) SELECT keyname, sum(keycount), date FROM events where date < ? group by date,keyname`,
        [strNow], function (err) {
          if (err) {
            return console.error(err.message);
          }
          let lines = this.changes
          console.log('转移数据条数: ', lines)
          // 删除 events 中的旧数据
          if (lines > 0) {
            db.run('delete FROM events where date < ? ', [strNow], function (err) {
              if (err) {
                return console.error(err.message);
              }
              console.log('删除数据条数: ', this.changes)
            });
          }
        });
    }
  });
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
function getDataSetting() {
  const db = new sqlite3.Database(dbName);
  return new Promise((resolve, reject) => {
    // 查询记录集
    let sql = `SELECT keyname,val from dataSetting2 
    union
    select 'mapDetail',mapDetail from keymaps join dataSetting2 on mapName = val where keyname='keymap' `
    db.all(sql, [], function (err, rows) {
      if (err) {
        reject(err);
      }
      // 输出记录集
      let hash = {}
      rows.forEach(r => {
        hash[r.keyname] = r.val
      })
      resolve(hash)
    });
    // 关闭数据库连接
    db.close();
  })
}
// 获取全部键盘
function getKeymaps() {
  const db = new sqlite3.Database(dbName);
  return new Promise((resolve, reject) => {
    // 查询记录集
    let arr = [];
    let sql = 'SELECT mapName,mapDetail FROM keymaps'
    db.all(sql, [], function (err, rows) {
      if (err) {
        reject(err);
      }
      // 输出记录集
      rows.forEach(function (row) {
        let mapName = row.mapName, mapDetail = row.mapDetail;
        arr.push({ mapName, mapDetail });
      });
      resolve(arr)
    });
    // 关闭数据库连接
    db.close();
  })
}
// 保存统计的相关配置
function setDataSetting(hash) {
  const db = new sqlite3.Database(dbName);
  return new Promise((resolve, reject) => {
    // 动态保存参数  mapDetail 参数在另外的表中，无需保存
    delete hash['mapDetail'];
    const placeholders = Object.keys(hash).map(() => '(?, ?)').join(', ');
    // 需要将内容全部按数组顺序排列
    const values = Object.entries(hash).flat(3); // Infinity 数组展开配置参数表
    db.run(`INSERT OR REPLACE INTO dataSetting2 (keyname, val) VALUES ${placeholders}`, values, function (err) {
      //let sql = 'INSERT OR REPLACE INTO keymap (key, value) VALUES'
      //db.all(sql, [hash.keymap, hash.screenSize, hash.mouseDPI, hash.topN], function (err, rows) {
      if (err) {
        reject(err);
      }
      // 输出记录集
      globalTopN = hash.topN
      globalAppTopN = hash.appTopN
      resolve(1)
    });
    // 关闭数据库连接
    db.close();
  })
}
// 操作 keymap , 0=删除 1=新增 2=更新
function optKeyMap(data) {
  console.log('optKeyMap:', data.mapName, data.flag)
  const db = new sqlite3.Database(dbName);
  return new Promise((resolve, reject) => {
    // 查询记录集
    let { mapName, mapDetail, flag } = data;
    if (mapName.toLowerCase() == 'default') {
      resolve(0)
      return
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
    db.all(sql, para, function (err, rows) {
      if (err) {
        reject(err);
      }
      // 输出记录集
      resolve(1)
    });
    // 关闭数据库连接
    db.close();
  })
}
// 获取全部历史天数
function getHistoryDate() {
  const db = new sqlite3.Database(dbName);
  return new Promise((resolve, reject) => {
    // 查询记录集
    let arr = [];
    let sql = 'SELECT date FROM stat group by date order by date desc'
    db.all(sql, [], function (err, rows) {
      if (err) {
        reject(err);
      }
      // 输出记录集
      rows.forEach(function (row) {
        arr.push(row.date);
      });
      resolve(arr)
    });
    // 关闭数据库连接
    db.close();
  })
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
    const placeholders = date.map(() => '?').join(',');
    if (flag == 0) {
      sql = 'delete FROM events where tick in(' + placeholders + ') '
    } else if (flag == 1) {
      sql = `delete FROM stat where date in(${placeholders}) ; 
      delete FROM statFreq where date in(${placeholders}) ;
      delete FROM appFreq where date in(${placeholders}) ;
      `  // 需要支持多个SQL
      date = date.concat(date, date);  // 有3个 placeholders 重复3次
    }
    console.log(sql, date)
    if (sql != '') {
      db.exec(sql, date, function (err) {
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
  return new Promise((resolve, reject) => {
    // 查询记录集
    let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name='dataSetting2';"
    db.all(sql, [], function (err, rows) {
      if (err) {
        reject(err);
      }
      // 输出记录集
      if (rows.length > 0) {
        resolve(0)
      } else {
        console.log('Need updateDBStruct')
        const db2 = new sqlite3.Database(dbName);
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
        db2.exec(sql, function (err) {  // 需要同时执行多条SQL
          if (err) {
            reject(err);
          }
          resolve(1)
        });
        db2.close();
      }
    });
    sql = "SELECT name FROM sqlite_master WHERE type='table' AND name='statFreq';"
    db.all(sql, [], function (err, rows) {
      if (err) {
        reject(err);
      }
      // 输出记录集
      if (rows.length > 0) {
        resolve(0)
      } else {
        const db2 = new sqlite3.Database(dbName);
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
          date TEXT
        );
        CREATE INDEX appFreq_date_IDX ON appFreq (date);
        `
        db2.exec(sql, function (err) {  // 需要同时执行多条SQL
          if (err) {
            reject(err);
          }
          resolve(1)
        });
        db2.close();
      }
    });
    // 关闭数据库连接
    db.close();
  })
}

// 获取最新的分钟信息
async function getLastMinute() {
  const db = new sqlite3.Database(dbName);
  return new Promise((resolve, reject) => {
    // 查询记录集
    let lastObj = {};
    let sql = 'SELECT keyTime,keyCount,mouseCount,distance FROM statFreq where freqType = 0 order by date desc, keyTime desc limit 1'
    db.all(sql, [], function (err, rows) {
      if (err) {
        reject(err);
      }
      rows.forEach(function (row) {
        lastObj = { "Distance": row.distance, "KeyCount": row.keyCount, "Minute": row.keyTime, "MouseCount": row.mouseCount };
      });
      resolve(lastObj)
    });
    db.close();
  })
}

module.exports = {
  insertData, getRecords, getDataSetting, setDataSetting, getKeymaps, optKeyMap, getHistoryDate,
  statData, deleteData, dbName, updateDBStruct, insertMiniute, getLastMinute
};