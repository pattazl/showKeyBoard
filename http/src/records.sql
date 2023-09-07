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
