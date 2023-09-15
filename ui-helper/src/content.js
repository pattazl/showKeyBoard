import { h } from 'vue';
import { NP, NText, NUl, NA, NPopover, NDivider, NOl } from 'naive-ui';

export default {
  'zh-CN': {
    title: '按键显示工具后台',
    lang: 'zh-CN',
    theme1:'深色',
    theme2:'浅色',
    menu:{
      setting:'设置',
      setting1:'个人喜好',
      setting2:'按键界面',
      setting3:'按键清单',
      stat: '数据',
      stat1: '今日数据',
      stat2: '历史分析',
      stat3: '数据趋势',
      stat4: '数据导出',
    },
    introTitle: '在你开始之前...',
    introOne: () => {
      return [
        h(NP, null, {
          default: () => [
            '首先，虽然我们为了中文用户的方便提供了中文的表单，但在填写时请',
            h(NText, { strong: true }, { default: () => '尽量使用英文' }),
            '。Naive UI 社区不只有中国人。如果你想让尽可能多的人能够看懂你的 issue，就请尽量用英文。如果担心表达不清楚，可以同时提供中文内容。',
          ],
        }),
        h(NP, null, {
          default: () => [
            '其次，Naive UI 及其相关库的 issue 列表只接受 bug 报告或是新功能请求 (feature requests)。这意味着我们不接受用法问题。如果你开的 issue 不符合规定，它将会被',
            h(NText, { strong: true }, { default: () => '立刻关闭' }),
            '。',
          ],
        }),
      ];
    },
    introWarningTitle: '不要用 Issue Helper 提使用问题！',
    introWarningContent: () => {
      return [
        '这只会让 Issue 被立即关闭。如果有使用问题，可以加入官方社区求助：',
        h(
          NPopover,
          {
            trigger: 'hover',
            style: 'padding: 6px 12px 12px 12px;',
          },
          {
            default: () => [
              h('div', { style: 'text-align: center;' }, [
                h(
                  NText,
                  { depth: 1, strong: true },
                  { default: () => '钉钉群 33482509' }
                ),
              ]),
              h(NDivider, {
                style: 'margin: 4px 0 12px -12px; width: calc(100% + 24px);',
              }),
              h('img', {
                style: {
                  width: '150px',
                  height: '150px',
                  display: 'block',
                },
                src: 'https://07akioni.oss-cn-beijing.aliyuncs.com/dingtalk-qr-code',
              }),
            ],
            trigger: () =>
              h(NA, null, {
                default: () => '钉钉交流群',
              }),
          }
        ),
        ' 或者 ',
        h(
          NA,
          {
            href: 'https://discord.com/invite/Pqv7Mev5Dd',
            target: '_blank',
          },
          {
            default: () => 'Discord',
          }
        ),
      ];
    },
    introTwo: () => {
      return [
        h(NP, null, {
          default: () => {
            return [
              '对于使用中遇到的问题，请使用以下资源：',
              h('br'),
              h(NUl, null, {
                default: () => {
                  return [
                    h(
                      'li',
                      null,
                      '仔细阅读文档：',
                      h(
                        NA,
                        {
                          href: 'https://www.naiveui.com',
                          target: '_blank',
                        },
                        {
                          default: () => 'Naive UI',
                        }
                      )
                    ),
                    h(
                      'li',
                      null,
                      '提问前确保你在 ',
                      h(
                        NA,
                        {
                          href: 'https://www.naiveui.com/zh-CN/os-theme/docs/common-issues',
                          target: '_blank',
                        },
                        {
                          default: () => '常见问题',
                        }
                      ),
                      ' 和 ',
                      h(
                        NA,
                        {
                          href: 'https://www.naiveui.com/zh-CN/os-theme/docs/changelog',
                          target: '_blank',
                        },
                        {
                          default: () => '更新日志',
                        }
                      ),
                      ' 中搜索过。'
                    ),
                  ];
                },
              }),
            ];
          },
        }),
        h(NP, null, {
          default: () =>
            '最后，在开 issue 前，可以先搜索一下以往的旧 issue — 你遇到的问题可能已经有人提了，也可能已经在最新版本中被修正。注意：如果你发现一个已经关闭的旧 issue 在最新版本中仍然存在，请不要在旧 issue 下面留言，而应该用下面的表单开一个新的 issue。',
        }),
      ];
    },
    explainTitle: '为什么要有这么严格的 issue 规定？',
    explain: () => [
      h(NP, null, {
        default: () => [
          '维护开源项目是',
          h(
            NA,
            {
              href: 'https://nolanlawson.com/2017/03/05/what-it-feels-like-to-be-an-open-source-maintainer/',
              target: '_blank',
            },
            { default: () => '非常辛苦的工作' }
          ),
          '。随着 Naive UI 在社区越来越受欢迎，我们每天都在收到越来越多的问题、bug 报告、功能需求和 Pull Requests。作为一个完全免费使用的开源项目，Naive UI 项目的维护人手是有限的。这意味着想要让项目长期的可持续发展，我们必须：',
        ],
      }),
      h(NOl, null, {
        default: () => [
          h(
            'li',
            null,
            '给予更具体的工作更高的优先级（比如 bug 的修复和新功能的开发）；'
          ),
          h('li', null, '提高 issue 处理的效率。'),
        ],
      }),
      h(NP, null, {
        default: () =>
          '针对（1），我们决定将 GitHub issue 列表严格地限制用于有具体目标和内容的工作。问题和讨论应当发送到更适合它们的场合。',
      }),
      h(NP, null, {
        default: () =>
          '针对（2），我们发现影响 issue 处理效率的最大因素是用户在开 issue 时没有提供足够的信息。这导致我们需要花费大量的时间去跟用户来回沟通，只为了获得一些基本信息好让我们对 issue 进行真正的分析。这正是我们开发 Issue Helper 的理由：我们要确保每个新 issue 都提供了必需的信息，这样能节省维护者和开发者双方的时间。',
      }),
      h(NP, null, {
        default: () =>
          '最重要的是，请明白一件事：开源项目的用户和维护者之间并不是甲方和乙方的关系，issue 也不是客服。在开 issue 的时候，请抱着一种『一起合作来解决这个问题』的心态，不要期待我们单方面地为你服务。',
      }),
    ],
    
  },
  'en-US': {
    title: 'Show Keyboard Server',
    lang: 'en-US',
    theme1:'Dark',
    theme2:'Light',
    menu:{
      setting:'Setting',
      setting1:'General',
      setting2:'Key UI',
      setting3:'KeyMap',
      stat: 'Data',
      stat1: 'Today',
      stat2: 'History',
      stat3: 'Statistics',
      stat4: 'Export',
    },
    introTitle: 'Before You Start...',
    introOne: () => {
      return h(NP, null, {
        default: () => [
          'The issue list is reserved exclusively for bug reports and feature requests. That means we do not accept usage questions. If you open an issue that does not conform to the requirements, ',
          h(
            NText,
            { strong: true },
            { default: () => 'it will be closed immediately' }
          ),
          '.',
        ],
      });
    },
    introWarningTitle: "Don't use Issue Helper to ask usage questions!",
    introWarningContent: () => {
      return [
        'This will only cause issue to be shut down immediately. If you have any problems, you can join the official community for help: ',
        h(
          NA,
          {
            href: 'https://discord.com/invite/Pqv7Mev5Dd',
            target: '_blank',
          },
          {
            default: () => 'Discord',
          }
        ),
        ' or ',
        h(
          NPopover,
          {
            trigger: 'hover',
            style: 'padding: 6px 12px 12px 12px;',
          },
          {
            default: () => [
              h('div', { style: 'text-align: center;' }, [
                h(
                  NText,
                  { depth: 1, strong: true },
                  { default: () => 'DingTalk Group' }
                ),
                h('br'),
                h(
                  NText,
                  { depth: 1, strong: true },
                  { default: () => '33482509' }
                ),
              ]),
              h(NDivider, {
                style: 'margin: 4px 0 12px -12px; width: calc(100% + 24px);',
              }),
              h('img', {
                style: {
                  width: '150px',
                  height: '150px',
                  display: 'block',
                },
                src: 'https://07akioni.oss-cn-beijing.aliyuncs.com/dingtalk-qr-code',
              }),
            ],
            trigger: () =>
              h(NA, null, {
                default: () => 'DingTalk Group',
              }),
          }
        ),
        '.',
      ];
    },
    introTwo: () => {
      return [
        h(NP, null, {
          default: () => {
            return [
              'For usage questions, please use the following resources:',
              h('br'),
              h(NUl, null, {
                default: () => {
                  return [
                    h(
                      'li',
                      null,
                      'Read the introduce and components documentation: ',
                      h(
                        NA,
                        {
                          href: 'https://www.naiveui.com',
                          target: '_blank',
                        },
                        {
                          default: () => 'Naive UI',
                        }
                      )
                    ),
                    h(
                      'li',
                      null,
                      'Make sure you have searched your question in ',
                      h(
                        NA,
                        {
                          href: 'https://www.naiveui.com/zh-CN/os-theme/docs/common-issues',
                          target: '_blank',
                        },
                        {
                          default: () => 'FAQ',
                        }
                      ),
                      ' and ',
                      h(
                        NA,
                        {
                          href: 'https://www.naiveui.com/zh-CN/os-theme/docs/changelog',
                          target: '_blank',
                        },
                        {
                          default: () => 'CHANGELOG',
                        }
                      )
                    ),
                  ];
                },
              }),
            ];
          },
        }),
        h(NP, null, {
          default: () =>
            'Also try to search for your issue - it may have already been answered or even fixed in the development branch. However, if you find that an old, closed issue still persists in the latest version, you should open a new issue using the form below instead of commenting on the old issue.',
        }),
      ];
    },
    explainTitle: 'Why are we so strict about this?',
    explain: () => [
      h(NP, null, {
        default: () => [
          'Maintaining open source projects, especially popular ones, is ',
          h(
            NA,
            {
              href: 'https://nolanlawson.com/2017/03/05/what-it-feels-like-to-be-an-open-source-maintainer/',
              target: '_blank',
            },
            { default: () => 'hard work' }
          ),
          ". As Naive UI's user base has grown, we are getting more and more usage questions, bug reports, feature requests and pull requests every single day. As a free and open source project, Naive UI also has limited maintainer bandwidth. That means the only way to ensure the project's sustainability is to:",
        ],
      }),
      h(NOl, null, {
        default: () => [
          h(
            'li',
            null,
            'Prioritize more concrete work (bug fixes and new features).'
          ),
          h('li', null, 'Improve issue triaging efficiency.'),
        ],
      }),
      h(NP, null, {
        default: () =>
          'For (1), we have decided to use the GitHub issue lists exclusively for work that has well-defined, actionable goals. Questions and open ended discussions should be posted to mediums that are better suited for them.',
      }),
      h(NP, null, {
        default: () =>
          'For (2), we have found that issues that do not provide proper information upfront usually results in terribly inefficient back-and-forth communication just to extract the basic information needed for actual triaging. This is exactly why we have created this app: to ensure that every issue is created with the necessary information, and to save time on both sides.',
      }),
    ]
  }
};
