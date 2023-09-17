import { h, Component, ref, toRef, PropType, computed ,watch } from 'vue'
import { NIcon} from 'naive-ui'
import type { MenuOption } from 'naive-ui'
import {
  BookOutline as BookIcon,
  PersonOutline as PersonIcon,
  SettingsOutline,
  HomeOutline,
  AnalyticsOutline,
  StatsChartOutline,
  KeypadOutline,
  SaveOutline,
  DesktopOutline,
  TodayOutline,
  TrendingUpOutline,
  DownloadOutline,
  BarChartOutline,
} from '@vicons/ionicons5'

function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) })
}
function setOpt(menuTxt){
  let menuOptions: MenuOption[] = [
    {
      label: menuTxt.home ,
      key: '/',
      icon: renderIcon(HomeOutline)
    },
      {
        //type: 'group',
        label: menuTxt.setting,
        key: menuTxt.setting,
        icon: renderIcon(SettingsOutline),
        children: [
          {
            label: menuTxt.setting1 ,
            key: '/Setting/General',
            icon: renderIcon(PersonIcon)
          },
          {
            label: menuTxt.setting2,
            key: '/Setting/KeyUI',
            icon: renderIcon(DesktopOutline)
          }, {
            label: menuTxt.setting3,
            key: '/Setting/StatPara',
            icon: renderIcon(StatsChartOutline)
          }, {
            label: menuTxt.setting4,
            key: '/Setting/KeyMap',
            icon: renderIcon(KeypadOutline)
          }, {
            label: menuTxt.setting5,
            key: '/Setting/Save',
            icon: renderIcon(SaveOutline)
          },
        ]
      },
      {
        //type: 'group',
        label: menuTxt.stat,
        key: menuTxt.stat,
        icon: renderIcon(BarChartOutline),
        children: [
          {
            label: menuTxt.stat1,
            key: '/Today',
            icon: renderIcon(TodayOutline)
          },
          {
            label: menuTxt.stat2,
            key: '/History',
            icon: renderIcon(BookIcon)
          }, {
            label: menuTxt.stat3,
            key: '/Statistics',
            icon: renderIcon(TrendingUpOutline)
          },{
            label: menuTxt.stat4,
            key: '/Export',
            icon: renderIcon(DownloadOutline)
          },
        ]
      }
    ]
    return menuOptions;
}
export {setOpt}

