/*
* @version 配置显示在登录页面的前端版本，格式为 v.主版本号.次版本号.日期.小版本序号 v1.0.190827.1
*/
// 对于发布版 YYMMDD 和 verNo 会在Webpack中自动替换为真实的日期和版本序号,如下格式不能随便改，参考vue.config.js的代码
const replaceYYMMDD = '231025';
const replaceVerNo = '1';
// 主版本从 package.json中获取
const mainVersion = '1.0';
const strVersion = mainVersion+'.'+replaceYYMMDD+'.'+replaceVerNo;   // 可以支持旧版浏览器
//export default strVersion
module.exports = {strVersion};
