//! emoji 别名表（FR-029，场景 7）。
//!
//! 内置常用 emoji 的拼音别名（`xiao`→😄），按 alias 字节序升序维护保证确定性；
//! 上限 512 条（T-062 已由首批 109 条扩至 381 条，仍在此内），查询用二分。
//! 字符均为 Unicode 标准字符，无外部数据源。

/// 别名表条目。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmojiEntry {
    /// 拼音/关键词别名（小写 ASCII 或中文）。
    pub alias: &'static str,
    /// 对应 emoji 字符。
    pub emoji: &'static str,
}

/// 首批高频别名表（按 alias 字节序升序；测试强制有序且无重复）。
pub static EMOJI_TABLE: &[EmojiEntry] = &[
    EmojiEntry {
        alias: "ai",
        emoji: "❤️",
    },
    EmojiEntry {
        alias: "aixin",
        emoji: "❤️",
    },
    EmojiEntry {
        alias: "baibai",
        emoji: "👋",
    },
    EmojiEntry {
        alias: "bangbangtang",
        emoji: "🍭",
    },
    EmojiEntry {
        alias: "bangqiu",
        emoji: "⚾",
    },
    EmojiEntry {
        alias: "banshou",
        emoji: "🔧",
    },
    EmojiEntry {
        alias: "baolingqiu",
        emoji: "🎳",
    },
    EmojiEntry {
        alias: "baoshi",
        emoji: "💎",
    },
    EmojiEntry {
        alias: "baozhi",
        emoji: "📰",
    },
    EmojiEntry {
        alias: "bi",
        emoji: "✏️",
    },
    EmojiEntry {
        alias: "bianfu",
        emoji: "🦇",
    },
    EmojiEntry {
        alias: "bianpao",
        emoji: "🧨",
    },
    EmojiEntry {
        alias: "bihu",
        emoji: "🦎",
    },
    EmojiEntry {
        alias: "biji",
        emoji: "📓",
    },
    EmojiEntry {
        alias: "binggan",
        emoji: "🍪",
    },
    EmojiEntry {
        alias: "bingqilin",
        emoji: "🍦",
    },
    EmojiEntry {
        alias: "bingqiu",
        emoji: "🏒",
    },
    EmojiEntry {
        alias: "bingshan",
        emoji: "🧊",
    },
    EmojiEntry {
        alias: "biye",
        emoji: "🎓",
    },
    EmojiEntry {
        alias: "bizui",
        emoji: "🤐",
    },
    EmojiEntry {
        alias: "boluo",
        emoji: "🍍",
    },
    EmojiEntry {
        alias: "boshi",
        emoji: "🎓",
    },
    EmojiEntry {
        alias: "cai",
        emoji: "🥬",
    },
    EmojiEntry {
        alias: "caidao",
        emoji: "🔪",
    },
    EmojiEntry {
        alias: "caihong",
        emoji: "🌈",
    },
    EmojiEntry {
        alias: "canting",
        emoji: "🍴",
    },
    EmojiEntry {
        alias: "cao",
        emoji: "🌿",
    },
    EmojiEntry {
        alias: "caomei",
        emoji: "🍓",
    },
    EmojiEntry {
        alias: "cha",
        emoji: "👎",
    },
    EmojiEntry {
        alias: "changjinglu",
        emoji: "🦒",
    },
    EmojiEntry {
        alias: "chaofan",
        emoji: "🍛",
    },
    EmojiEntry {
        alias: "chaoxiao",
        emoji: "😝",
    },
    EmojiEntry {
        alias: "chatou",
        emoji: "🔌",
    },
    EmojiEntry {
        alias: "che",
        emoji: "🚗",
    },
    EmojiEntry {
        alias: "chengbao",
        emoji: "🏰",
    },
    EmojiEntry {
        alias: "chengzi",
        emoji: "🍊",
    },
    EmojiEntry {
        alias: "chizi",
        emoji: "📏",
    },
    EmojiEntry {
        alias: "chongbai",
        emoji: "🤩",
    },
    EmojiEntry {
        alias: "chonglang",
        emoji: "🏄",
    },
    EmojiEntry {
        alias: "chuan",
        emoji: "🚢",
    },
    EmojiEntry {
        alias: "chuizi",
        emoji: "🔨",
    },
    EmojiEntry {
        alias: "chuzuche",
        emoji: "🚕",
    },
    EmojiEntry {
        alias: "cipan",
        emoji: "🍽️",
    },
    EmojiEntry {
        alias: "ciwei",
        emoji: "🦔",
    },
    EmojiEntry {
        alias: "cuo",
        emoji: "❌",
    },
    EmojiEntry {
        alias: "dan",
        emoji: "🥚",
    },
    EmojiEntry {
        alias: "danche",
        emoji: "🚲",
    },
    EmojiEntry {
        alias: "dangao",
        emoji: "🎂",
    },
    EmojiEntry {
        alias: "danta",
        emoji: "🥧",
    },
    EmojiEntry {
        alias: "daxiao",
        emoji: "😄",
    },
    EmojiEntry {
        alias: "daxingxing",
        emoji: "🦍",
    },
    EmojiEntry {
        alias: "deng",
        emoji: "💡",
    },
    EmojiEntry {
        alias: "dengyan",
        emoji: "😳",
    },
    EmojiEntry {
        alias: "dianchi",
        emoji: "🔋",
    },
    EmojiEntry {
        alias: "dianhua",
        emoji: "📞",
    },
    EmojiEntry {
        alias: "diannao",
        emoji: "💻",
    },
    EmojiEntry {
        alias: "dianshi",
        emoji: "📺",
    },
    EmojiEntry {
        alias: "dianying",
        emoji: "🎬",
    },
    EmojiEntry {
        alias: "diaoyu",
        emoji: "🎣",
    },
    EmojiEntry {
        alias: "diqiu",
        emoji: "🌍",
    },
    EmojiEntry {
        alias: "ditie",
        emoji: "🚇",
    },
    EmojiEntry {
        alias: "ditu",
        emoji: "🗺️",
    },
    EmojiEntry {
        alias: "dizhi",
        emoji: "📍",
    },
    EmojiEntry {
        alias: "dui",
        emoji: "✅",
    },
    EmojiEntry {
        alias: "duijiangji",
        emoji: "📟",
    },
    EmojiEntry {
        alias: "e",
        emoji: "🦢",
    },
    EmojiEntry {
        alias: "erji",
        emoji: "🎧",
    },
    EmojiEntry {
        alias: "eyu",
        emoji: "🐊",
    },
    EmojiEntry {
        alias: "fangdajing",
        emoji: "🔍",
    },
    EmojiEntry {
        alias: "fangzi",
        emoji: "🏠",
    },
    EmojiEntry {
        alias: "fannao",
        emoji: "😩",
    },
    EmojiEntry {
        alias: "feibiao",
        emoji: "🎯",
    },
    EmojiEntry {
        alias: "feiji",
        emoji: "✈️",
    },
    EmojiEntry {
        alias: "feng",
        emoji: "🍃",
    },
    EmojiEntry {
        alias: "fengzheng",
        emoji: "🪁",
    },
    EmojiEntry {
        alias: "fennu",
        emoji: "😡",
    },
    EmojiEntry {
        alias: "fenxin",
        emoji: "💗",
    },
    EmojiEntry {
        alias: "futou",
        emoji: "🪓",
    },
    EmojiEntry {
        alias: "gali",
        emoji: "🍛",
    },
    EmojiEntry {
        alias: "ganbei",
        emoji: "🥂",
    },
    EmojiEntry {
        alias: "gangqin",
        emoji: "🎹",
    },
    EmojiEntry {
        alias: "gangua",
        emoji: "😳",
    },
    EmojiEntry {
        alias: "ganlanqiu",
        emoji: "🏈",
    },
    EmojiEntry {
        alias: "gantan",
        emoji: "❗",
    },
    EmojiEntry {
        alias: "ganxie",
        emoji: "🙏",
    },
    EmojiEntry {
        alias: "gaoerfu",
        emoji: "⛳",
    },
    EmojiEntry {
        alias: "gequ",
        emoji: "🎶",
    },
    EmojiEntry {
        alias: "glasses",
        emoji: "👓",
    },
    EmojiEntry {
        alias: "gongjiaoche",
        emoji: "🚌",
    },
    EmojiEntry {
        alias: "gou",
        emoji: "🐶",
    },
    EmojiEntry {
        alias: "gu",
        emoji: "🥁",
    },
    EmojiEntry {
        alias: "gua",
        emoji: "🐸",
    },
    EmojiEntry {
        alias: "guanjun",
        emoji: "🏆",
    },
    EmojiEntry {
        alias: "guoshanche",
        emoji: "🎢",
    },
    EmojiEntry {
        alias: "guzhang",
        emoji: "👏",
    },
    EmojiEntry {
        alias: "haha",
        emoji: "😂",
    },
    EmojiEntry {
        alias: "hai",
        emoji: "🌊",
    },
    EmojiEntry {
        alias: "haibao",
        emoji: "🦭",
    },
    EmojiEntry {
        alias: "haixiu",
        emoji: "😳",
    },
    EmojiEntry {
        alias: "hama",
        emoji: "🐸",
    },
    EmojiEntry {
        alias: "hamigua",
        emoji: "🍈",
    },
    EmojiEntry {
        alias: "hanbao",
        emoji: "🍔",
    },
    EmojiEntry {
        alias: "hehe",
        emoji: "😁",
    },
    EmojiEntry {
        alias: "heixin",
        emoji: "🖤",
    },
    EmojiEntry {
        alias: "hema",
        emoji: "🦛",
    },
    EmojiEntry {
        alias: "hongbao",
        emoji: "🧧",
    },
    EmojiEntry {
        alias: "hongdeng",
        emoji: "🚦",
    },
    EmojiEntry {
        alias: "hongjiu",
        emoji: "🍷",
    },
    EmojiEntry {
        alias: "hongxin",
        emoji: "❤️",
    },
    EmojiEntry {
        alias: "houzi",
        emoji: "🐵",
    },
    EmojiEntry {
        alias: "hua",
        emoji: "🌸",
    },
    EmojiEntry {
        alias: "huanggua",
        emoji: "🥒",
    },
    EmojiEntry {
        alias: "huangguan",
        emoji: "👑",
    },
    EmojiEntry {
        alias: "huangxin",
        emoji: "💛",
    },
    EmojiEntry {
        alias: "huasheng",
        emoji: "🥜",
    },
    EmojiEntry {
        alias: "huaxue",
        emoji: "⛷️",
    },
    EmojiEntry {
        alias: "huaxueban",
        emoji: "🏂",
    },
    EmojiEntry {
        alias: "huaye",
        emoji: "🍂",
    },
    EmojiEntry {
        alias: "huixing",
        emoji: "☄️",
    },
    EmojiEntry {
        alias: "huli",
        emoji: "🦊",
    },
    EmojiEntry {
        alias: "huluobo",
        emoji: "🥕",
    },
    EmojiEntry {
        alias: "huo",
        emoji: "🔥",
    },
    EmojiEntry {
        alias: "huoche",
        emoji: "🚂",
    },
    EmojiEntry {
        alias: "huochezhan",
        emoji: "🚉",
    },
    EmojiEntry {
        alias: "huoji",
        emoji: "🦃",
    },
    EmojiEntry {
        alias: "huojian",
        emoji: "🚀",
    },
    EmojiEntry {
        alias: "huolieniao",
        emoji: "🦩",
    },
    EmojiEntry {
        alias: "huoshan",
        emoji: "🌋",
    },
    EmojiEntry {
        alias: "ji",
        emoji: "🐔",
    },
    EmojiEntry {
        alias: "jia",
        emoji: "🏠",
    },
    EmojiEntry {
        alias: "jian",
        emoji: "✂️",
    },
    EmojiEntry {
        alias: "jiangbei",
        emoji: "🏆",
    },
    EmojiEntry {
        alias: "jiangpai",
        emoji: "🥇",
    },
    EmojiEntry {
        alias: "jianpan",
        emoji: "⌨️",
    },
    EmojiEntry {
        alias: "jianshen",
        emoji: "🏋️",
    },
    EmojiEntry {
        alias: "jiaotang",
        emoji: "⛪",
    },
    EmojiEntry {
        alias: "jiaozi",
        emoji: "🥟",
    },
    EmojiEntry {
        alias: "jiayou",
        emoji: "💪",
    },
    EmojiEntry {
        alias: "jichang",
        emoji: "🛫",
    },
    EmojiEntry {
        alias: "jiezhi",
        emoji: "💍",
    },
    EmojiEntry {
        alias: "jijian",
        emoji: "🤺",
    },
    EmojiEntry {
        alias: "jinbi",
        emoji: "🪙",
    },
    EmojiEntry {
        alias: "jingche",
        emoji: "🚓",
    },
    EmojiEntry {
        alias: "jinggao",
        emoji: "⚠️",
    },
    EmojiEntry {
        alias: "jingyu",
        emoji: "🐳",
    },
    EmojiEntry {
        alias: "jingzi",
        emoji: "🪞",
    },
    EmojiEntry {
        alias: "jinzhi",
        emoji: "🚫",
    },
    EmojiEntry {
        alias: "jiqiren",
        emoji: "🤖",
    },
    EmojiEntry {
        alias: "jitui",
        emoji: "🍗",
    },
    EmojiEntry {
        alias: "jiudian",
        emoji: "🏨",
    },
    EmojiEntry {
        alias: "jiuhuche",
        emoji: "🚑",
    },
    EmojiEntry {
        alias: "juzhong",
        emoji: "🏋️",
    },
    EmojiEntry {
        alias: "kache",
        emoji: "🚚",
    },
    EmojiEntry {
        alias: "kafei",
        emoji: "☕",
    },
    EmojiEntry {
        alias: "keai",
        emoji: "🥰",
    },
    EmojiEntry {
        alias: "konglong",
        emoji: "🦕",
    },
    EmojiEntry {
        alias: "kongqiqiu",
        emoji: "🎈",
    },
    EmojiEntry {
        alias: "kouhong",
        emoji: "💄",
    },
    EmojiEntry {
        alias: "ku",
        emoji: "😂",
    },
    EmojiEntry {
        alias: "kuaidi",
        emoji: "📦",
    },
    EmojiEntry {
        alias: "kuaizi",
        emoji: "🥢",
    },
    EmojiEntry {
        alias: "kulu",
        emoji: "💀",
    },
    EmojiEntry {
        alias: "kuqi",
        emoji: "😭",
    },
    EmojiEntry {
        alias: "kuxiao",
        emoji: "😅",
    },
    EmojiEntry {
        alias: "lajiao",
        emoji: "🌶️",
    },
    EmojiEntry {
        alias: "lamian",
        emoji: "🍜",
    },
    EmojiEntry {
        alias: "lang",
        emoji: "🐺",
    },
    EmojiEntry {
        alias: "lanqiu",
        emoji: "🏀",
    },
    EmojiEntry {
        alias: "lanxin",
        emoji: "💙",
    },
    EmojiEntry {
        alias: "laohu",
        emoji: "🐯",
    },
    EmojiEntry {
        alias: "laoshu",
        emoji: "🐭",
    },
    EmojiEntry {
        alias: "lapa",
        emoji: "📢",
    },
    EmojiEntry {
        alias: "lazhu",
        emoji: "🕯️",
    },
    EmojiEntry {
        alias: "leng",
        emoji: "🥶",
    },
    EmojiEntry {
        alias: "lenghan",
        emoji: "💦",
    },
    EmojiEntry {
        alias: "li",
        emoji: "🍐",
    },
    EmojiEntry {
        alias: "libao",
        emoji: "🎁",
    },
    EmojiEntry {
        alias: "lingdang",
        emoji: "🔔",
    },
    EmojiEntry {
        alias: "liubing",
        emoji: "⛸️",
    },
    EmojiEntry {
        alias: "liulei",
        emoji: "😢",
    },
    EmojiEntry {
        alias: "liuxing",
        emoji: "🌠",
    },
    EmojiEntry {
        alias: "liwu",
        emoji: "🎁",
    },
    EmojiEntry {
        alias: "long",
        emoji: "🐲",
    },
    EmojiEntry {
        alias: "longjuanfeng",
        emoji: "🌪️",
    },
    EmojiEntry {
        alias: "lu",
        emoji: "🦌",
    },
    EmojiEntry {
        alias: "luosidao",
        emoji: "🪛",
    },
    EmojiEntry {
        alias: "luotuo",
        emoji: "🐫",
    },
    EmojiEntry {
        alias: "lvxin",
        emoji: "💚",
    },
    EmojiEntry {
        alias: "lvxing",
        emoji: "✈️",
    },
    EmojiEntry {
        alias: "ma",
        emoji: "🐴",
    },
    EmojiEntry {
        alias: "mangguo",
        emoji: "🥭",
    },
    EmojiEntry {
        alias: "mao",
        emoji: "🐱",
    },
    EmojiEntry {
        alias: "maotouying",
        emoji: "🦉",
    },
    EmojiEntry {
        alias: "maozi",
        emoji: "🎩",
    },
    EmojiEntry {
        alias: "maque",
        emoji: "🐦",
    },
    EmojiEntry {
        alias: "mashu",
        emoji: "🏇",
    },
    EmojiEntry {
        alias: "mayi",
        emoji: "🐜",
    },
    EmojiEntry {
        alias: "meigui",
        emoji: "🌹",
    },
    EmojiEntry {
        alias: "mianbao",
        emoji: "🍞",
    },
    EmojiEntry {
        alias: "miantiao",
        emoji: "🍜",
    },
    EmojiEntry {
        alias: "mianyang",
        emoji: "🐑",
    },
    EmojiEntry {
        alias: "mifan",
        emoji: "🍚",
    },
    EmojiEntry {
        alias: "mifeng",
        emoji: "🐝",
    },
    EmojiEntry {
        alias: "mogu",
        emoji: "🍄",
    },
    EmojiEntry {
        alias: "mogui",
        emoji: "👿",
    },
    EmojiEntry {
        alias: "motianlun",
        emoji: "🎡",
    },
    EmojiEntry {
        alias: "motoche",
        emoji: "🏍️",
    },
    EmojiEntry {
        alias: "mozhang",
        emoji: "🪄",
    },
    EmojiEntry {
        alias: "muma",
        emoji: "😘",
    },
    EmojiEntry {
        alias: "naicha",
        emoji: "🧋",
    },
    EmojiEntry {
        alias: "nanguo",
        emoji: "😞",
    },
    EmojiEntry {
        alias: "naozhong",
        emoji: "⏰",
    },
    EmojiEntry {
        alias: "niao",
        emoji: "🐦",
    },
    EmojiEntry {
        alias: "ningmeng",
        emoji: "🍋",
    },
    EmojiEntry {
        alias: "niu",
        emoji: "🐮",
    },
    EmojiEntry {
        alias: "niunai",
        emoji: "🥛",
    },
    EmojiEntry {
        alias: "ok",
        emoji: "👌",
    },
    EmojiEntry {
        alias: "pai",
        emoji: "🥧",
    },
    EmojiEntry {
        alias: "paiqiu",
        emoji: "🏐",
    },
    EmojiEntry {
        alias: "pangxie",
        emoji: "🦀",
    },
    EmojiEntry {
        alias: "paobu",
        emoji: "🏃",
    },
    EmojiEntry {
        alias: "peigen",
        emoji: "🥓",
    },
    EmojiEntry {
        alias: "pengyou",
        emoji: "🤗",
    },
    EmojiEntry {
        alias: "piaochong",
        emoji: "🐞",
    },
    EmojiEntry {
        alias: "pijiu",
        emoji: "🍺",
    },
    EmojiEntry {
        alias: "pingguo",
        emoji: "🍎",
    },
    EmojiEntry {
        alias: "pingpangqiu",
        emoji: "🏓",
    },
    EmojiEntry {
        alias: "pisa",
        emoji: "🍕",
    },
    EmojiEntry {
        alias: "puke",
        emoji: "🃏",
    },
    EmojiEntry {
        alias: "putao",
        emoji: "🍇",
    },
    EmojiEntry {
        alias: "qian",
        emoji: "💰",
    },
    EmojiEntry {
        alias: "qianbao",
        emoji: "👛",
    },
    EmojiEntry {
        alias: "qianbi",
        emoji: "✏️",
    },
    EmojiEntry {
        alias: "qianzhang",
        emoji: "🙌",
    },
    EmojiEntry {
        alias: "qiaokeli",
        emoji: "🍫",
    },
    EmojiEntry {
        alias: "qidao",
        emoji: "🙏",
    },
    EmojiEntry {
        alias: "qie",
        emoji: "🐧",
    },
    EmojiEntry {
        alias: "qiezi",
        emoji: "🍆",
    },
    EmojiEntry {
        alias: "qingwa",
        emoji: "🐸",
    },
    EmojiEntry {
        alias: "qinqin",
        emoji: "😘",
    },
    EmojiEntry {
        alias: "qiu",
        emoji: "⚽",
    },
    EmojiEntry {
        alias: "qiyiguo",
        emoji: "🥝",
    },
    EmojiEntry {
        alias: "quantou",
        emoji: "👊",
    },
    EmojiEntry {
        alias: "qugunqiu",
        emoji: "🏑",
    },
    EmojiEntry {
        alias: "re",
        emoji: "🥵",
    },
    EmojiEntry {
        alias: "regou",
        emoji: "🌭",
    },
    EmojiEntry {
        alias: "reqiqiu",
        emoji: "🎈",
    },
    EmojiEntry {
        alias: "rili",
        emoji: "📅",
    },
    EmojiEntry {
        alias: "riqi",
        emoji: "📅",
    },
    EmojiEntry {
        alias: "sangshi",
        emoji: "🧟",
    },
    EmojiEntry {
        alias: "sanmingzhi",
        emoji: "🥪",
    },
    EmojiEntry {
        alias: "sax",
        emoji: "🎷",
    },
    EmojiEntry {
        alias: "shala",
        emoji: "🥗",
    },
    EmojiEntry {
        alias: "shan",
        emoji: "⛰️",
    },
    EmojiEntry {
        alias: "shandian",
        emoji: "⚡",
    },
    EmojiEntry {
        alias: "shang",
        emoji: "⬆️",
    },
    EmojiEntry {
        alias: "shangdian",
        emoji: "🏪",
    },
    EmojiEntry {
        alias: "shanyang",
        emoji: "🐐",
    },
    EmojiEntry {
        alias: "shayu",
        emoji: "🦈",
    },
    EmojiEntry {
        alias: "she",
        emoji: "🐍",
    },
    EmojiEntry {
        alias: "shejian",
        emoji: "🏹",
    },
    EmojiEntry {
        alias: "shengdanlaoren",
        emoji: "🎅",
    },
    EmojiEntry {
        alias: "shenghao",
        emoji: "🦪",
    },
    EmojiEntry {
        alias: "shengji",
        emoji: "🆙",
    },
    EmojiEntry {
        alias: "shengli",
        emoji: "✌️",
    },
    EmojiEntry {
        alias: "shengqi",
        emoji: "😡",
    },
    EmojiEntry {
        alias: "shengri",
        emoji: "🎂",
    },
    EmojiEntry {
        alias: "shijian",
        emoji: "⏰",
    },
    EmojiEntry {
        alias: "shizi",
        emoji: "🦁",
    },
    EmojiEntry {
        alias: "shoubiao",
        emoji: "⌚",
    },
    EmojiEntry {
        alias: "shoudiantong",
        emoji: "🔦",
    },
    EmojiEntry {
        alias: "shouji",
        emoji: "📱",
    },
    EmojiEntry {
        alias: "shousi",
        emoji: "🍣",
    },
    EmojiEntry {
        alias: "shu",
        emoji: "📖",
    },
    EmojiEntry {
        alias: "shuaxin",
        emoji: "🔄",
    },
    EmojiEntry {
        alias: "shubiao",
        emoji: "🖱️",
    },
    EmojiEntry {
        alias: "shui",
        emoji: "😴",
    },
    EmojiEntry {
        alias: "shuizhu",
        emoji: "💧",
    },
    EmojiEntry {
        alias: "shuye",
        emoji: "🍂",
    },
    EmojiEntry {
        alias: "sikao",
        emoji: "🤔",
    },
    EmojiEntry {
        alias: "simiao",
        emoji: "🛕",
    },
    EmojiEntry {
        alias: "songshu",
        emoji: "🐿️",
    },
    EmojiEntry {
        alias: "soup",
        emoji: "🍲",
    },
    EmojiEntry {
        alias: "suo",
        emoji: "🔒",
    },
    EmojiEntry {
        alias: "taidixiong",
        emoji: "🧸",
    },
    EmojiEntry {
        alias: "taifeng",
        emoji: "🌀",
    },
    EmojiEntry {
        alias: "taiquandao",
        emoji: "🥋",
    },
    EmojiEntry {
        alias: "taiyang",
        emoji: "☀️",
    },
    EmojiEntry {
        alias: "tang",
        emoji: "🍬",
    },
    EmojiEntry {
        alias: "tanshou",
        emoji: "🤷",
    },
    EmojiEntry {
        alias: "tiane",
        emoji: "🦢",
    },
    EmojiEntry {
        alias: "tianqi",
        emoji: "🌤️",
    },
    EmojiEntry {
        alias: "tianshi",
        emoji: "😇",
    },
    EmojiEntry {
        alias: "tiantianquan",
        emoji: "🍩",
    },
    EmojiEntry {
        alias: "ticao",
        emoji: "🤸",
    },
    EmojiEntry {
        alias: "tiqin",
        emoji: "🎻",
    },
    EmojiEntry {
        alias: "tu",
        emoji: "🐰",
    },
    EmojiEntry {
        alias: "tudou",
        emoji: "🥔",
    },
    EmojiEntry {
        alias: "tuolaji",
        emoji: "🚜",
    },
    EmojiEntry {
        alias: "tuoniao",
        emoji: "🦤",
    },
    EmojiEntry {
        alias: "tuse",
        emoji: "😝",
    },
    EmojiEntry {
        alias: "tushuguan",
        emoji: "📚",
    },
    EmojiEntry {
        alias: "waixingren",
        emoji: "👽",
    },
    EmojiEntry {
        alias: "wan'an",
        emoji: "🌙",
    },
    EmojiEntry {
        alias: "wangqiu",
        emoji: "🎾",
    },
    EmojiEntry {
        alias: "wei",
        emoji: "⚠️",
    },
    EmojiEntry {
        alias: "weixiao",
        emoji: "😊",
    },
    EmojiEntry {
        alias: "wenhao",
        emoji: "❓",
    },
    EmojiEntry {
        alias: "wenquan",
        emoji: "♨️",
    },
    EmojiEntry {
        alias: "woniu",
        emoji: "🐌",
    },
    EmojiEntry {
        alias: "woshou",
        emoji: "🤝",
    },
    EmojiEntry {
        alias: "wu",
        emoji: "🌫️",
    },
    EmojiEntry {
        alias: "wugui",
        emoji: "🐢",
    },
    EmojiEntry {
        alias: "wuliao",
        emoji: "😑",
    },
    EmojiEntry {
        alias: "wunai",
        emoji: "😮",
    },
    EmojiEntry {
        alias: "xia",
        emoji: "🦐",
    },
    EmojiEntry {
        alias: "xiang",
        emoji: "🐘",
    },
    EmojiEntry {
        alias: "xiangbin",
        emoji: "🍾",
    },
    EmojiEntry {
        alias: "xiangchang",
        emoji: "🌭",
    },
    EmojiEntry {
        alias: "xiangji",
        emoji: "📷",
    },
    EmojiEntry {
        alias: "xiangjiao",
        emoji: "🍌",
    },
    EmojiEntry {
        alias: "xiangxia",
        emoji: "⬇️",
    },
    EmojiEntry {
        alias: "xiangyi",
        emoji: "🤔",
    },
    EmojiEntry {
        alias: "xianhua",
        emoji: "💐",
    },
    EmojiEntry {
        alias: "xiao",
        emoji: "😄",
    },
    EmojiEntry {
        alias: "xiaofangche",
        emoji: "🚒",
    },
    EmojiEntry {
        alias: "xiaolaba",
        emoji: "🎺",
    },
    EmojiEntry {
        alias: "xiaoyu",
        emoji: "🌧️",
    },
    EmojiEntry {
        alias: "xin",
        emoji: "💕",
    },
    EmojiEntry {
        alias: "xingxing",
        emoji: "⭐",
    },
    EmojiEntry {
        alias: "xiniu",
        emoji: "🦏",
    },
    EmojiEntry {
        alias: "xinlv",
        emoji: "💓",
    },
    EmojiEntry {
        alias: "xinnian",
        emoji: "🎉",
    },
    EmojiEntry {
        alias: "xinsui",
        emoji: "💔",
    },
    EmojiEntry {
        alias: "xiong",
        emoji: "🐻",
    },
    EmojiEntry {
        alias: "xiongmao",
        emoji: "🐼",
    },
    EmojiEntry {
        alias: "xishuai",
        emoji: "🦗",
    },
    EmojiEntry {
        alias: "xue",
        emoji: "❄️",
    },
    EmojiEntry {
        alias: "xuegao",
        emoji: "🍦",
    },
    EmojiEntry {
        alias: "xuehua",
        emoji: "❄️",
    },
    EmojiEntry {
        alias: "xueren",
        emoji: "⛄",
    },
    EmojiEntry {
        alias: "xuexiao",
        emoji: "🏫",
    },
    EmojiEntry {
        alias: "ya",
        emoji: "🦆",
    },
    EmojiEntry {
        alias: "yangcong",
        emoji: "🧅",
    },
    EmojiEntry {
        alias: "yanhua",
        emoji: "🎆",
    },
    EmojiEntry {
        alias: "yanjing",
        emoji: "👀",
    },
    EmojiEntry {
        alias: "yaoshi",
        emoji: "🔑",
    },
    EmojiEntry {
        alias: "yifu",
        emoji: "👕",
    },
    EmojiEntry {
        alias: "ying",
        emoji: "🦅",
    },
    EmojiEntry {
        alias: "yinger",
        emoji: "👶",
    },
    EmojiEntry {
        alias: "yingtao",
        emoji: "🍒",
    },
    EmojiEntry {
        alias: "yinhang",
        emoji: "🏦",
    },
    EmojiEntry {
        alias: "yinle",
        emoji: "🎵",
    },
    EmojiEntry {
        alias: "yisheng",
        emoji: "🧑‍⚕️",
    },
    EmojiEntry {
        alias: "yiyuan",
        emoji: "🏥",
    },
    EmojiEntry {
        alias: "you",
        emoji: "➡️",
    },
    EmojiEntry {
        alias: "youlechang",
        emoji: "🎢",
    },
    EmojiEntry {
        alias: "youling",
        emoji: "👻",
    },
    EmojiEntry {
        alias: "youxiji",
        emoji: "🎮",
    },
    EmojiEntry {
        alias: "youyong",
        emoji: "🏊",
    },
    EmojiEntry {
        alias: "yu",
        emoji: "🐟",
    },
    EmojiEntry {
        alias: "yueliang",
        emoji: "🌙",
    },
    EmojiEntry {
        alias: "yumaoqiu",
        emoji: "🏸",
    },
    EmojiEntry {
        alias: "yumi",
        emoji: "🌽",
    },
    EmojiEntry {
        alias: "yun",
        emoji: "☁️",
    },
    EmojiEntry {
        alias: "yunfan",
        emoji: "⛵",
    },
    EmojiEntry {
        alias: "yusan",
        emoji: "☂️",
    },
    EmojiEntry {
        alias: "zan",
        emoji: "👍",
    },
    EmojiEntry {
        alias: "zhadan",
        emoji: "💣",
    },
    EmojiEntry {
        alias: "zhinanzhen",
        emoji: "🧭",
    },
    EmojiEntry {
        alias: "zhishengji",
        emoji: "🚁",
    },
    EmojiEntry {
        alias: "zhizhu",
        emoji: "🕷️",
    },
    EmojiEntry {
        alias: "zhong",
        emoji: "🎯",
    },
    EmojiEntry {
        alias: "zhu",
        emoji: "🐷",
    },
    EmojiEntry {
        alias: "zhuakuang",
        emoji: "🤯",
    },
    EmojiEntry {
        alias: "zixin",
        emoji: "💜",
    },
    EmojiEntry {
        alias: "zixingche",
        emoji: "🚲",
    },
    EmojiEntry {
        alias: "zuanshi",
        emoji: "💎",
    },
    EmojiEntry {
        alias: "zui",
        emoji: "💋",
    },
    EmojiEntry {
        alias: "zuo",
        emoji: "⬅️",
    },
    EmojiEntry {
        alias: "zuoyu",
        emoji: "🐡",
    },
];

/// 按别名查询 emoji；未命中返回 `None`（大小写敏感，别名均为小写）。
#[must_use]
pub fn emoji_for(alias: &str) -> Option<&'static str> {
    EMOJI_TABLE
        .binary_search_by(|entry| entry.alias.cmp(alias))
        .ok()
        .map(|index| EMOJI_TABLE[index].emoji)
}

#[cfg(test)]
mod tests {
    use super::{emoji_for, EMOJI_TABLE};
    use std::collections::HashSet;

    #[test]
    fn 表有序且无重复别名() {
        let mut seen: HashSet<&str> = HashSet::new();
        for pair in EMOJI_TABLE.windows(2) {
            assert!(
                pair[0].alias < pair[1].alias,
                "表未按 alias 升序: {} vs {}",
                pair[0].alias,
                pair[1].alias
            );
        }
        for entry in EMOJI_TABLE {
            assert!(seen.insert(entry.alias), "重复别名: {}", entry.alias);
            assert!(!entry.alias.is_empty() && !entry.emoji.is_empty());
        }
    }

    #[test]
    fn 表规模在约束内() {
        assert!(
            EMOJI_TABLE.len() >= 300,
            "T-062 应 ≥300 条，实际 {}",
            EMOJI_TABLE.len()
        );
        assert!(EMOJI_TABLE.len() <= 512, "上限 512 条");
    }

    #[test]
    fn 验收别名命中() {
        assert_eq!(emoji_for("xiao"), Some("😄"));
        assert_eq!(emoji_for("ku"), Some("😂"));
        assert_eq!(emoji_for("aixin"), Some("❤️"));
        assert_eq!(emoji_for("shui"), Some("😴"));
        assert_eq!(emoji_for("zan"), Some("👍"));
        assert_eq!(emoji_for("ok"), Some("👌"));
        // T-062 扩展批量命中
        assert_eq!(emoji_for("biye"), Some("🎓"));
        assert_eq!(emoji_for("she"), Some("🐍"));
        assert_eq!(emoji_for("xishuai"), Some("🦗"));
        assert_eq!(emoji_for("soup"), Some("🍲"));
        assert_eq!(emoji_for("glasses"), Some("👓"));
        assert_eq!(emoji_for("xiongmao"), Some("🐼"));
        assert_eq!(emoji_for("hanbao"), Some("🍔"));
        assert_eq!(emoji_for("taidixiong"), Some("🧸"));
        assert_eq!(emoji_for("mozhang"), Some("🪄"));
        assert_eq!(emoji_for("yanjing"), Some("👀"));
        assert_eq!(emoji_for("tang"), Some("🍬"));
    }

    #[test]
    fn 不命中返回空() {
        assert_eq!(emoji_for("xyzabc"), None);
        assert_eq!(emoji_for(""), None);
        assert_eq!(emoji_for("XIAO"), None);
        assert_eq!(emoji_for("xiao "), None);
    }
}
