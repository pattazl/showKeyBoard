const dayjs = require('dayjs');
const sqlite3 = require('sqlite3').verbose();

let strDate = dayjs(new Date()).format('YYYY-MM-DD')
// 创建一个连接到数据库的对象
// 创建表格（如果不存在）
function insertData(records) {
  return new Promise((resolve, reject) => {
    const db = new sqlite3.Database('records.db');
    let tick = records['tick']
    // 先检查是否已经有此tick数据，如果有则删除
    db.run('delete FROM events where tick = ? ', [tick], function (err) {
      if (err) {
        return console.error(err.message);
      }
      console.log('删除tick数据条数: ', this.changes )
    });
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
    const values = arr.reduce((acc, curr) => acc.concat([curr.name, curr.count, curr.tick, strDate]), []);
    // 执行一次性插入
    console.log('insert into sql')
    db.run(`INSERT INTO events (keyname, keycount , tick , date) VALUES ${placeholders}`, values, function (err) {
      if (err) {
        console.error(err.message);
        reject(err)
      }
      console.log(`A total of ${this.changes} rows have been inserted`);
      resolve(this.changes)
    });
    // 关闭数据库连接
    db.close();
  })
}


function getRecords(begin, end) {
  const db = new sqlite3.Database('records.db');
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
      sql = 'SELECT keyname, keycount, date FROM stat where date between ? and ? '
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
    // 检查 events 是否有小于当天的数据,如果有，则需要触发转移
    if (begin == strNow && end == strNow) {
      db.all('SELECT tick FROM events where date<? limit 1', [strNow], function (err, rows) {
        if (err) {
          //reject(err);
        }
        // 输出记录集
        if (rows.length > 0) {
          setTimeout(doCleanData, 1);
        }
      });
    }
    // 关闭数据库连接
    db.close();
  })
}
// 将 events的旧数据统计后转移到stat 表中
function doCleanData() {
  const db = new sqlite3.Database('records.db');
  console.log('doCleanData')
  let strNow = dayjs(new Date()).format('YYYY-MM-DD')
  // 执行sql处理，插入统计数据
  db.run(`INSERT INTO stat (keyname, keycount,date) SELECT keyname, sum(keycount), date FROM events where date < ? group by date,keyname`,
    [strNow], function (err) {
      if (err) {
        return console.error(err.message);
      }
      let lines = this.changes
      console.log('转移数据条数: ',lines)
      // 删除 events 中的旧数据
      if (lines > 0) {
        db.run('delete FROM events where date < ? ', [strNow], function (err) {
          if (err) {
            return console.error(err.message);
          }
          console.log('删除数据条数: ', this.changes )
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

module.exports = {
  insertData, getRecords
};