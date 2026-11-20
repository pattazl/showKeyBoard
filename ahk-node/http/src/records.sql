-- 当前的事件记录，如果数据在stat表中，则删除
CREATE TABLE events (
	keyname TEXT,  -- 键名
	keycount INTEGER, -- 按键次数
	date TEXT,   -- 日期
	tick INTEGER   -- 时间戳
);
CREATE INDEX events_date_IDX ON events (date);
CREATE INDEX events_tick_IDX ON events (tick);

-- 统计信息，对多个tick进行叠加计算后的数据
CREATE TABLE stat (
	keyname TEXT,  -- 键名
	keycount INTEGER, -- 按键次数
	date TEXT,   -- 日期
);
CREATE INDEX stat_date_IDX ON stat (date);
CREATE INDEX stat_keyname_IDX ON stat (keyname);

CREATE TABLE "keymaps" (
  "mapName" TEXT NOT NULL,
  "mapDetail" text
);

CREATE TABLE "dataSetting" (
  "keymap" TEXT,
  "screenSize" integer,
  "mouseDPI" integer
);
CREATE TABLE dataSetting2(
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
  select 'topN' as keyname , topN as  val  from dataSetting;
-- 使用的频繁程度，保存每分钟和每小时的统计信息
CREATE TABLE statFreq (
    keyTime TEXT,  -- 日期和时间
    keyCount INTEGER, -- 按键次数
    mouseCount INTEGER, -- 鼠标次数
    distance INTEGER,   -- 鼠标距离
    freqType INTEGER,   -- 0=分钟，1=小时
    date TEXT   -- 具体日期，用于方便分析统计
);
CREATE INDEX statFreq_date_IDX ON statFreq (date);
CREATE INDEX statFreq_type_IDX ON statFreq (freqType);
-- 应用每分钟使用频率的表
CREATE TABLE appFreq (
  keyTime TEXT,   -- 日期和时间
  appPath TEXT,   -- 应用路径
  keyCount INTEGER, -- 按键次数
  mouseCount INTEGER, -- 鼠标次数
  freqType INTEGER,   -- 0=分钟，>0 表示该小时内的占用的分钟数
  date TEXT    -- 具体日期，用于方便分析统计
);
CREATE INDEX appFreq_date_IDX ON appFreq (date);
CREATE INDEX appFreq_type_IDX ON appFreq (freqType);
