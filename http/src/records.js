const dayjs = require('dayjs');
const sqlite3 = require('sqlite3').verbose();

// 创建一个连接到数据库的对象
// 创建表格（如果不存在）
function insertData(records) {
  let strDate = dayjs(new Date()).format('YYYY-MM-DD')
  return new Promise((resolve, reject) => {
    const db = new sqlite3.Database('records.db');
    let tick = records['tick']
	if(tick == null ){
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
    db.all('SELECT tick FROM events where date<? limit 1', [strDate], function (err, rows) {
      if (err) {
        //reject(err);
      }
      // 输出记录集
      if (rows.length > 0) {
        setTimeout(doCleanData, 5000); // 过5秒后再处理，因为上面一行代码可能还在执行
      }
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
  db.close();
}
/*
(async ()=>{
  arr = await getRecords()
  console.log(arr)
})()
*/
// 数据库中的相关配置信息，主要用于统计
function getDataSetting() {
  const db = new sqlite3.Database('records.db');
  return new Promise((resolve, reject) => {
    // 查询记录集
    let sql = 'SELECT keymap,mapDetail,screenSize,mouseDPI FROM dataSetting left join keymaps on keymap = mapname '
    db.all(sql, [], function (err, rows) {
      if (err) {
        reject(err);
      }
      // 输出记录集
	  if(rows.length>0){
		  row = rows[0]
		  keymap = row.keymap
		  mapDetail = row.mapDetail
		  screenSize = row.screenSize
		  mouseDPI = row.mouseDPI
		  resolve({keymap,mapDetail,screenSize,mouseDPI})
	  }else{
		  resolve({})
	  }
    });
    // 关闭数据库连接
    db.close();
  })
}
// 获取全部键盘
function getKeymaps() {
  const db = new sqlite3.Database('records.db');
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
  const db = new sqlite3.Database('records.db');
  return new Promise((resolve, reject) => {
    // 查询记录集
    let sql = 'update dataSetting set keymap = ? ,screenSize = ? ,mouseDPI = ? '
    db.all(sql, [hash.keymap,hash.screenSize,hash.mouseDPI], function (err, rows) {
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
// 操作 keymap , 0=删除 1=新增 2=更新
function optKeyMap(data) {
  console.log('optKeyMap:',data.mapName,data.flag)
  const db = new sqlite3.Database('records.db');
  return new Promise((resolve, reject) => {
    // 查询记录集
	let {mapName,mapDetail,flag} = data;
	if( mapName.toLowerCase()=='default'){
		resolve(0)
		return
	}
	let sql = '',para = []
	switch(flag)
	{
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
    db.all(sql, para , function (err, rows) {
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
  const db = new sqlite3.Database('records.db');
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
async function statData(begin,end) {
  const db = new sqlite3.Database('records.db');
  let pro1 =new Promise((resolve, reject) => {
    // 查询鼠标移动距离
    let sql = "select keycount,date from stat where date between ? and ? and keyname = 'mouseDistance' order by date"
    db.all(sql, [begin,end], function (err, rows) {
      if (err) {
        reject(err);
      }
      resolve(rows)
    });
  })
  // 查询鼠标点击数量和按键数量总和
  let pro2 =new Promise((resolve, reject) => {
    // 查询记录集
    let sql = `select sum(keycount) as keycount,date,'mouse' as keyname from stat 
    where date between ? and ? and keyname in ('LButton','RButton','MButton','WheelDown','WheelUp') group by date
UNION
select sum(keycount) as keycount,date,'keyboard' from stat 
where date between ? and ? and keyname not in ('mouseDistance','LButton','RButton','MButton','WheelDown','WheelUp') group by date`
    db.all(sql, [begin,end,begin,end], function (err, rows) {
      if (err) {
        reject(err);
      }
      resolve(rows);
    });
  })
  // 查询统计范围内按键数 top10
  let pro3 =new Promise((resolve, reject) => {
    // 查询记录集
    let sql = `select keycount,date,keyname from stat where keyname in
(select keyname from stat where date between ? and ? and keyname not in ('mouseDistance','LButton','RButton','MButton','WheelDown','WheelUp') group by keyname order by sum(keycount) desc limit 15
) and date between ? and ? order by date`
    db.all(sql, [begin,end,begin,end], function (err, rows) {
      if (err) {
        reject(err);
      }
      resolve(rows)
    });
  })
  let arr = await Promise.all([pro1,pro2,pro3]);
  db.close();
  return arr;
}


module.exports = {
  insertData, getRecords,getDataSetting,setDataSetting,getKeymaps,optKeyMap,getHistoryDate,statData
};