const dayjs = require('dayjs');
const sqlite3 = require('sqlite3').verbose();

let strDate = dayjs(new Date()).format('YYYY-MM-DD')
// 创建一个连接到数据库的对象
const db = new sqlite3.Database('records.db');

// 创建表格（如果不存在）
function insertData(records) {
  return new Promise((resolve, reject) => {
    // 将record 中的所有键值 插入数据
    let arr = []
    let tick = records['tick']
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
    db.run(`INSERT INTO events (keyname, keycount , tick , date) VALUES ${placeholders}`, values, function (err) {
      if (err) {
        //return console.error(err.message);
        reject(err)
      }
      //console.log(`A total of ${this.changes} rows have been inserted`);
      resolve(this.changes)
    });
    // 关闭数据库连接
    db.close();
  })
}

function getRecords() {
  return new Promise((resolve, reject) => {
    // 查询记录集
    let arr = [];
    db.all('SELECT * FROM events where date = ?', [strDate], function (err, rows) {
      if (err) {
        reject(err);
      }
      // 输出记录集
      rows.forEach(function (row) {
        let keyname = row.keyname, keycount = row.keycount, date = row.date;
        arr.push({ keyname, keycount, date });
      });
      resolve(arr)
    });
    // 关闭数据库连接
    db.close();
  })
}
(async ()=>{
  arr = await getRecords()
  console.log(arr)
})()
