//! 单字拼音注音表（通用规范汉字表读音，kTGHZ2013）。
//!
//! 本文件由 scripts/build-char-pinyin.ps1 从 data/cache/kTGHZ2013.txt 生成，
//! 不得手改；变更需改脚本后重跑（输入哈希由 data/pins/pinyin-data-kTGHZ2013.json 锁定）。
//!
//! 供场景 9 通讯录（FR-036/FR-037）联系人姓名注音建键：返回某字**全部**读音形态
//! （无调 ASCII，ü 按输入法惯例写作 v），多音字任一读音均可命中（D-22）。

/// 单条字-读音表项（按字标量序升序，二分查询）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharPinyinEntry {
    /// 汉字。
    pub ch: char,
    /// 全部读音形态（无调，ü->v）。
    pub readings: &'static [&'static str],
}

/// 单字注音表（kTGHZ2013，经脚本清洗）。
pub static CHAR_PINYIN: &[CharPinyinEntry] = &[
    CharPinyinEntry {
        ch: '\u{3447}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{344A}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{356E}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{360E}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{364D}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{3658}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{3666}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{36C3}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{36DA}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{36F9}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{37C3}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{3807}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{3813}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{3918}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{3944}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{39D0}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{39D1}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{39DF}',
        readings: &["kuai"],
    },
    CharPinyinEntry {
        ch: '\u{3AF0}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{3B0A}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{3B0E}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{3B1A}',
        readings: &["che"],
    },
    CharPinyinEntry {
        ch: '\u{3B4E}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{3B55}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{3BBE}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{3C00}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{3CC7}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{3CD8}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{3CDA}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{3D14}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{3D50}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{3DB2}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{3E06}',
        readings: &["kao"],
    },
    CharPinyinEntry {
        ch: '\u{3E0C}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{3E84}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{3EEC}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{3F4F}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{3FE0}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{4056}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{40AE}',
        readings: &["lve"],
    },
    CharPinyinEntry {
        ch: '\u{40C5}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{40CE}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{415F}',
        readings: &["can"],
    },
    CharPinyinEntry {
        ch: '\u{4339}',
        readings: &["jiong"],
    },
    CharPinyinEntry {
        ch: '\u{4383}',
        readings: &["ran"],
    },
    CharPinyinEntry {
        ch: '\u{4396}',
        readings: &["zeng"],
    },
    CharPinyinEntry {
        ch: '\u{43DD}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{43E1}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{43F2}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{4403}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{44D6}',
        readings: &["qiong"],
    },
    CharPinyinEntry {
        ch: '\u{44DB}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{44E8}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{44EB}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{44EC}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{45D6}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{45DB}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{45EA}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{45F4}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{4723}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{4759}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{48BA}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{48BC}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{48D8}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{497D}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{4983}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{4C9F}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{4CA0}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{4CA2}',
        readings: &["teng"],
    },
    CharPinyinEntry {
        ch: '\u{4D13}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{4D14}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{4D15}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{4D16}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{4D17}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{4D18}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{4D19}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{4DAE}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{4E00}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4E01}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{4E03}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{4E07}',
        readings: &["mo", "wan"],
    },
    CharPinyinEntry {
        ch: '\u{4E08}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{4E09}',
        readings: &["san"],
    },
    CharPinyinEntry {
        ch: '\u{4E0A}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{4E0B}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{4E0D}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{4E0E}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{4E0F}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{4E10}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{4E11}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{4E13}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{4E14}',
        readings: &["qie"],
    },
    CharPinyinEntry {
        ch: '\u{4E15}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{4E16}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{4E18}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{4E19}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{4E1A}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{4E1B}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{4E1C}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{4E1D}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{4E1E}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{4E22}',
        readings: &["diu"],
    },
    CharPinyinEntry {
        ch: '\u{4E24}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{4E25}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{4E27}',
        readings: &["sang"],
    },
    CharPinyinEntry {
        ch: '\u{4E2A}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{4E2B}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{4E2D}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{4E30}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{4E32}',
        readings: &["chuan"],
    },
    CharPinyinEntry {
        ch: '\u{4E34}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{4E38}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{4E39}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{4E3A}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{4E3B}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{4E3D}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{4E3E}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{4E42}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4E43}',
        readings: &["nai"],
    },
    CharPinyinEntry {
        ch: '\u{4E45}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{4E48}',
        readings: &["me"],
    },
    CharPinyinEntry {
        ch: '\u{4E49}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4E4B}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{4E4C}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{4E4D}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{4E4E}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{4E4F}',
        readings: &["fa"],
    },
    CharPinyinEntry {
        ch: '\u{4E50}',
        readings: &["le", "yue"],
    },
    CharPinyinEntry {
        ch: '\u{4E52}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{4E53}',
        readings: &["pang"],
    },
    CharPinyinEntry {
        ch: '\u{4E54}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{4E56}',
        readings: &["guai"],
    },
    CharPinyinEntry {
        ch: '\u{4E58}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{4E59}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4E5C}',
        readings: &["mie", "nie"],
    },
    CharPinyinEntry {
        ch: '\u{4E5D}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{4E5E}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{4E5F}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{4E60}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{4E61}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{4E66}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{4E69}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{4E70}',
        readings: &["mai"],
    },
    CharPinyinEntry {
        ch: '\u{4E71}',
        readings: &["luan"],
    },
    CharPinyinEntry {
        ch: '\u{4E73}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{4E78}',
        readings: &["na"],
    },
    CharPinyinEntry {
        ch: '\u{4E7E}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{4E86}',
        readings: &["le", "liao"],
    },
    CharPinyinEntry {
        ch: '\u{4E88}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{4E89}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{4E8B}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{4E8C}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{4E8D}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{4E8E}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{4E8F}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{4E91}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{4E92}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{4E93}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{4E94}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{4E95}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{4E98}',
        readings: &["gen"],
    },
    CharPinyinEntry {
        ch: '\u{4E9A}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{4E9B}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{4E9F}',
        readings: &["ji", "qi"],
    },
    CharPinyinEntry {
        ch: '\u{4EA1}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{4EA2}',
        readings: &["kang"],
    },
    CharPinyinEntry {
        ch: '\u{4EA4}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{4EA5}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{4EA6}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4EA7}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{4EA8}',
        readings: &["heng"],
    },
    CharPinyinEntry {
        ch: '\u{4EA9}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{4EAB}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{4EAC}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{4EAD}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{4EAE}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{4EB2}',
        readings: &["qin", "qing"],
    },
    CharPinyinEntry {
        ch: '\u{4EB3}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{4EB5}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{4EB6}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{4EB8}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{4EB9}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{4EBA}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{4EBF}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4EC0}',
        readings: &["shen", "shi"],
    },
    CharPinyinEntry {
        ch: '\u{4EC1}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{4EC2}',
        readings: &["le"],
    },
    CharPinyinEntry {
        ch: '\u{4EC3}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{4EC4}',
        readings: &["ze"],
    },
    CharPinyinEntry {
        ch: '\u{4EC5}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{4EC6}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{4EC7}',
        readings: &["chou", "qiu"],
    },
    CharPinyinEntry {
        ch: '\u{4EC9}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{4ECA}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{4ECB}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{4ECD}',
        readings: &["reng"],
    },
    CharPinyinEntry {
        ch: '\u{4ECE}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{4ED1}',
        readings: &["lun"],
    },
    CharPinyinEntry {
        ch: '\u{4ED3}',
        readings: &["cang"],
    },
    CharPinyinEntry {
        ch: '\u{4ED4}',
        readings: &["zai", "zi"],
    },
    CharPinyinEntry {
        ch: '\u{4ED5}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{4ED6}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{4ED7}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{4ED8}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{4ED9}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{4EDD}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{4EDE}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{4EDF}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{4EE1}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{4EE3}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{4EE4}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{4EE5}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4EE8}',
        readings: &["sa"],
    },
    CharPinyinEntry {
        ch: '\u{4EEA}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4EEB}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{4EEC}',
        readings: &["men"],
    },
    CharPinyinEntry {
        ch: '\u{4EF0}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{4EF2}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{4EF3}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{4EF5}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{4EF6}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{4EF7}',
        readings: &["jia", "jie"],
    },
    CharPinyinEntry {
        ch: '\u{4EFB}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{4EFD}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{4EFF}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{4F01}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{4F08}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{4F09}',
        readings: &["kang"],
    },
    CharPinyinEntry {
        ch: '\u{4F0A}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4F0B}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{4F0D}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{4F0E}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{4F0F}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{4F10}',
        readings: &["fa"],
    },
    CharPinyinEntry {
        ch: '\u{4F11}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{4F17}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{4F18}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{4F19}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{4F1A}',
        readings: &["hui", "kuai"],
    },
    CharPinyinEntry {
        ch: '\u{4F1B}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{4F1E}',
        readings: &["san"],
    },
    CharPinyinEntry {
        ch: '\u{4F1F}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{4F20}',
        readings: &["chuan", "zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{4F22}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{4F23}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{4F24}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{4F25}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{4F26}',
        readings: &["lun"],
    },
    CharPinyinEntry {
        ch: '\u{4F27}',
        readings: &["cang"],
    },
    CharPinyinEntry {
        ch: '\u{4F2A}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{4F2B}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{4F2D}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{4F2F}',
        readings: &["bai", "bo"],
    },
    CharPinyinEntry {
        ch: '\u{4F30}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{4F32}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{4F34}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{4F36}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{4F38}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{4F3A}',
        readings: &["ci", "si"],
    },
    CharPinyinEntry {
        ch: '\u{4F3C}',
        readings: &["shi", "si"],
    },
    CharPinyinEntry {
        ch: '\u{4F3D}',
        readings: &["ga", "jia", "qie"],
    },
    CharPinyinEntry {
        ch: '\u{4F3E}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{4F41}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4F43}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{4F46}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{4F4D}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{4F4E}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{4F4F}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{4F50}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{4F51}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{4F53}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{4F55}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{4F56}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{4F57}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{4F58}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{4F59}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{4F5A}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4F5B}',
        readings: &["fo", "fu"],
    },
    CharPinyinEntry {
        ch: '\u{4F5C}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{4F5D}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{4F5E}',
        readings: &["ning"],
    },
    CharPinyinEntry {
        ch: '\u{4F5F}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{4F60}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{4F63}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{4F64}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{4F65}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{4F69}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{4F6C}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{4F6F}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{4F70}',
        readings: &["bai"],
    },
    CharPinyinEntry {
        ch: '\u{4F73}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{4F74}',
        readings: &["nai"],
    },
    CharPinyinEntry {
        ch: '\u{4F76}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{4F78}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{4F7A}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{4F7B}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{4F7C}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{4F7D}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{4F7E}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4F7F}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{4F81}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{4F82}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{4F83}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{4F84}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{4F88}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{4F89}',
        readings: &["kua"],
    },
    CharPinyinEntry {
        ch: '\u{4F8B}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{4F8D}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{4F8F}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{4F91}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{4F94}',
        readings: &["mou"],
    },
    CharPinyinEntry {
        ch: '\u{4F97}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{4F98}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{4F9B}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{4F9D}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{4FA0}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{4FA3}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{4FA5}',
        readings: &["jiao", "yao"],
    },
    CharPinyinEntry {
        ch: '\u{4FA6}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{4FA7}',
        readings: &["ce"],
    },
    CharPinyinEntry {
        ch: '\u{4FA8}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{4FA9}',
        readings: &["kuai"],
    },
    CharPinyinEntry {
        ch: '\u{4FAA}',
        readings: &["chai"],
    },
    CharPinyinEntry {
        ch: '\u{4FAC}',
        readings: &["nong"],
    },
    CharPinyinEntry {
        ch: '\u{4FAE}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{4FAF}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{4FB4}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{4FB5}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{4FB9}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{4FBF}',
        readings: &["bian", "pian"],
    },
    CharPinyinEntry {
        ch: '\u{4FC3}',
        readings: &["cu"],
    },
    CharPinyinEntry {
        ch: '\u{4FC4}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{4FC5}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{4FCA}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{4FCD}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{4FCE}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{4FCF}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{4FD0}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{4FD1}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{4FD7}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{4FD8}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{4FD9}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{4FDA}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{4FDC}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{4FDD}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{4FDE}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{4FDF}',
        readings: &["qi", "si"],
    },
    CharPinyinEntry {
        ch: '\u{4FE1}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{4FE3}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{4FE6}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{4FE8}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{4FE9}',
        readings: &["lia", "liang"],
    },
    CharPinyinEntry {
        ch: '\u{4FEA}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{4FEB}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{4FED}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{4FEE}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{4FEF}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{4FF1}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{4FF3}',
        readings: &["pai"],
    },
    CharPinyinEntry {
        ch: '\u{4FF5}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{4FF6}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{4FF8}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{4FFA}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{4FFE}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{500C}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{500D}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{500F}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{5012}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{5013}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{5014}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{5015}',
        readings: &["chui"],
    },
    CharPinyinEntry {
        ch: '\u{5018}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{5019}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{501A}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{501C}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{501E}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{501F}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{5021}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{5025}',
        readings: &["kong"],
    },
    CharPinyinEntry {
        ch: '\u{5026}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{5027}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{5028}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{5029}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{502A}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{502C}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{502D}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{502E}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{5034}',
        readings: &["ben"],
    },
    CharPinyinEntry {
        ch: '\u{503A}',
        readings: &["zhai"],
    },
    CharPinyinEntry {
        ch: '\u{503B}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{503C}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{503E}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{5041}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{5043}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5047}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{5048}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{504C}',
        readings: &["ruo"],
    },
    CharPinyinEntry {
        ch: '\u{504E}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{504F}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{5053}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{5055}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{505A}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{505C}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{5061}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{5065}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{506C}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{506D}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{5070}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{5072}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{5076}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{5077}',
        readings: &["tou"],
    },
    CharPinyinEntry {
        ch: '\u{507B}',
        readings: &["lou", "lv"],
    },
    CharPinyinEntry {
        ch: '\u{507E}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{507F}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{5080}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{5083}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{5085}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5088}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{5089}',
        readings: &["nu"],
    },
    CharPinyinEntry {
        ch: '\u{508D}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{5092}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5095}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{50A3}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{50A5}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{50A7}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{50A8}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{50A9}',
        readings: &["nuo"],
    },
    CharPinyinEntry {
        ch: '\u{50AC}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{50B2}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{50BA}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{50BB}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{50C7}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{50CE}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{50CF}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{50D4}',
        readings: &["zun"],
    },
    CharPinyinEntry {
        ch: '\u{50D6}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{50DA}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{50E6}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{50E7}',
        readings: &["seng"],
    },
    CharPinyinEntry {
        ch: '\u{50EC}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{50ED}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{50EE}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{50F0}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{50F3}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{50F5}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{50FB}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{5106}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{5107}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{510B}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{5112}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{5121}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{5126}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{5133}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{5134}',
        readings: &["rang"],
    },
    CharPinyinEntry {
        ch: '\u{513F}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{5140}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5141}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{5143}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{5144}',
        readings: &["xiong"],
    },
    CharPinyinEntry {
        ch: '\u{5145}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{5146}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{5148}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{5149}',
        readings: &["guang"],
    },
    CharPinyinEntry {
        ch: '\u{514B}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{514D}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{5151}',
        readings: &["dui"],
    },
    CharPinyinEntry {
        ch: '\u{5154}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{5155}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{5156}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{515A}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{515C}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{5162}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{5165}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{5168}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{516B}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{516C}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{516D}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{516E}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5170}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{5171}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{5173}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{5174}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{5175}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{5176}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{5177}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{5178}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{5179}',
        readings: &["ci", "zi"],
    },
    CharPinyinEntry {
        ch: '\u{517B}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{517C}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{517D}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{5180}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5181}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{5185}',
        readings: &["nei"],
    },
    CharPinyinEntry {
        ch: '\u{5188}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{5189}',
        readings: &["ran"],
    },
    CharPinyinEntry {
        ch: '\u{518C}',
        readings: &["ce"],
    },
    CharPinyinEntry {
        ch: '\u{518D}',
        readings: &["zai"],
    },
    CharPinyinEntry {
        ch: '\u{518F}',
        readings: &["jiong"],
    },
    CharPinyinEntry {
        ch: '\u{5192}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{5194}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{5195}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{5197}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{5199}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{519B}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{519C}',
        readings: &["nong"],
    },
    CharPinyinEntry {
        ch: '\u{51A0}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{51A2}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{51A4}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{51A5}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{51AC}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{51AE}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{51AF}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{51B0}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{51B1}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{51B2}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{51B3}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{51B5}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{51B6}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{51B7}',
        readings: &["leng"],
    },
    CharPinyinEntry {
        ch: '\u{51BB}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{51BC}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{51BD}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{51C0}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{51C4}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{51C6}',
        readings: &["zhun"],
    },
    CharPinyinEntry {
        ch: '\u{51C7}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{51C9}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{51CB}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{51CC}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{51CF}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{51D1}',
        readings: &["cou"],
    },
    CharPinyinEntry {
        ch: '\u{51D3}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{51D8}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{51DB}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{51DD}',
        readings: &["ning"],
    },
    CharPinyinEntry {
        ch: '\u{51E0}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{51E1}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{51E4}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{51EB}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{51ED}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{51EF}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{51F0}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{51F3}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{51F6}',
        readings: &["xiong"],
    },
    CharPinyinEntry {
        ch: '\u{51F8}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{51F9}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{51FA}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{51FB}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{51FC}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{51FD}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{51FF}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{5200}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{5201}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{5203}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{5206}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{5207}',
        readings: &["qie"],
    },
    CharPinyinEntry {
        ch: '\u{5208}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{520A}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{520D}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{520E}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{5211}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{5212}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{5216}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{5217}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{5218}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{5219}',
        readings: &["ze"],
    },
    CharPinyinEntry {
        ch: '\u{521A}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{521B}',
        readings: &["chuang"],
    },
    CharPinyinEntry {
        ch: '\u{521D}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{5220}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{5224}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{5228}',
        readings: &["bao", "pao"],
    },
    CharPinyinEntry {
        ch: '\u{5229}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{522B}',
        readings: &["bie"],
    },
    CharPinyinEntry {
        ch: '\u{522C}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{522D}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{522E}',
        readings: &["gua"],
    },
    CharPinyinEntry {
        ch: '\u{5230}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{5233}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{5236}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{5237}',
        readings: &["shua"],
    },
    CharPinyinEntry {
        ch: '\u{5238}',
        readings: &["quan", "xuan"],
    },
    CharPinyinEntry {
        ch: '\u{5239}',
        readings: &["cha", "sha"],
    },
    CharPinyinEntry {
        ch: '\u{523A}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{523B}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{523D}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{523F}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{5240}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{5241}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{5242}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5243}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{5245}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{524A}',
        readings: &["xiao", "xue"],
    },
    CharPinyinEntry {
        ch: '\u{524B}',
        readings: &["kei"],
    },
    CharPinyinEntry {
        ch: '\u{524C}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{524D}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{5250}',
        readings: &["gua"],
    },
    CharPinyinEntry {
        ch: '\u{5251}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{5254}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{5255}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{5256}',
        readings: &["pou"],
    },
    CharPinyinEntry {
        ch: '\u{525C}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{525E}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{525F}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{5261}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{5265}',
        readings: &["bao", "bo"],
    },
    CharPinyinEntry {
        ch: '\u{5267}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{5269}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{526A}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{526F}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5272}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{527D}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{527F}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{5281}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{5282}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{5284}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{5288}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{5290}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{5293}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{529B}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{529D}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{529E}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{529F}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{52A0}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{52A1}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{52A2}',
        readings: &["mai"],
    },
    CharPinyinEntry {
        ch: '\u{52A3}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{52A8}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{52A9}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{52AA}',
        readings: &["nu"],
    },
    CharPinyinEntry {
        ch: '\u{52AB}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{52AC}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{52AD}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{52B1}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{52B2}',
        readings: &["jin", "jing"],
    },
    CharPinyinEntry {
        ch: '\u{52B3}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{52BC}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{52BE}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{52BF}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{52C3}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{52C7}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{52C9}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{52CB}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{52CD}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{52D0}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{52D2}',
        readings: &["le", "lei"],
    },
    CharPinyinEntry {
        ch: '\u{52D4}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{52D6}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{52D8}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{52DA}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{52DF}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{52E0}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{52E4}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{52F0}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{52FA}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{52FE}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{52FF}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5300}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{5305}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{5306}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{5308}',
        readings: &["xiong"],
    },
    CharPinyinEntry {
        ch: '\u{530D}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{530F}',
        readings: &["pao"],
    },
    CharPinyinEntry {
        ch: '\u{5310}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5315}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5316}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{5317}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{5319}',
        readings: &["chi", "shi"],
    },
    CharPinyinEntry {
        ch: '\u{531C}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{531D}',
        readings: &["za"],
    },
    CharPinyinEntry {
        ch: '\u{5320}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{5321}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{5323}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{5326}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{532A}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{532E}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{5339}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{533A}',
        readings: &["ou", "qu"],
    },
    CharPinyinEntry {
        ch: '\u{533B}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{533C}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{533E}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{533F}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{5341}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5343}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{5345}',
        readings: &["sa"],
    },
    CharPinyinEntry {
        ch: '\u{5347}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{5348}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5349}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{534A}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{534E}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{534F}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{5351}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{5352}',
        readings: &["cu", "zu"],
    },
    CharPinyinEntry {
        ch: '\u{5353}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{5355}',
        readings: &["chan", "dan", "shan"],
    },
    CharPinyinEntry {
        ch: '\u{5356}',
        readings: &["mai"],
    },
    CharPinyinEntry {
        ch: '\u{5357}',
        readings: &["na", "nan"],
    },
    CharPinyinEntry {
        ch: '\u{535A}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{535C}',
        readings: &["bo", "bu"],
    },
    CharPinyinEntry {
        ch: '\u{535E}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{535F}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{5360}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{5361}',
        readings: &["ka", "qia"],
    },
    CharPinyinEntry {
        ch: '\u{5362}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{5363}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{5364}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{5366}',
        readings: &["gua"],
    },
    CharPinyinEntry {
        ch: '\u{5367}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{536B}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{536C}',
        readings: &["ang"],
    },
    CharPinyinEntry {
        ch: '\u{536E}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{536F}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{5370}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{5371}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5373}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5374}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{5375}',
        readings: &["luan"],
    },
    CharPinyinEntry {
        ch: '\u{5377}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{5378}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{537A}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{537F}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{5382}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{5384}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{5385}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{5386}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{5389}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{538B}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{538C}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{538D}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{5395}',
        readings: &["ce"],
    },
    CharPinyinEntry {
        ch: '\u{5396}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{5398}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{539A}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{539D}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{539F}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{53A2}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{53A3}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{53A5}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{53A6}',
        readings: &["sha", "xia"],
    },
    CharPinyinEntry {
        ch: '\u{53A8}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{53A9}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{53AE}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{53BB}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{53BE}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{53BF}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{53C1}',
        readings: &["san"],
    },
    CharPinyinEntry {
        ch: '\u{53C2}',
        readings: &["can", "cen", "shen"],
    },
    CharPinyinEntry {
        ch: '\u{53C6}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{53C7}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{53C8}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{53C9}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{53CA}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{53CB}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{53CC}',
        readings: &["shuang"],
    },
    CharPinyinEntry {
        ch: '\u{53CD}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{53D1}',
        readings: &["fa"],
    },
    CharPinyinEntry {
        ch: '\u{53D4}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{53D5}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{53D6}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{53D7}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{53D8}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{53D9}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{53DA}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{53DB}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{53DF}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{53E0}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{53E3}',
        readings: &["kou"],
    },
    CharPinyinEntry {
        ch: '\u{53E4}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{53E5}',
        readings: &["gou", "ju"],
    },
    CharPinyinEntry {
        ch: '\u{53E6}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{53E8}',
        readings: &["dao", "tao"],
    },
    CharPinyinEntry {
        ch: '\u{53E9}',
        readings: &["kou"],
    },
    CharPinyinEntry {
        ch: '\u{53EA}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{53EB}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{53EC}',
        readings: &["shao", "zhao"],
    },
    CharPinyinEntry {
        ch: '\u{53ED}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{53EE}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{53EF}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{53F0}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{53F1}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{53F2}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{53F3}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{53F5}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{53F6}',
        readings: &["xie", "ye"],
    },
    CharPinyinEntry {
        ch: '\u{53F7}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{53F8}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{53F9}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{53FB}',
        readings: &["le"],
    },
    CharPinyinEntry {
        ch: '\u{53FC}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{53FD}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5401}',
        readings: &["xu", "yu"],
    },
    CharPinyinEntry {
        ch: '\u{5403}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{5404}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{5406}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{5408}',
        readings: &["ge", "he"],
    },
    CharPinyinEntry {
        ch: '\u{5409}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{540A}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{540C}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{540D}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{540E}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{540F}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{5410}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{5411}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{5412}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{5413}',
        readings: &["he", "xia"],
    },
    CharPinyinEntry {
        ch: '\u{5415}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{5416}',
        readings: &["a"],
    },
    CharPinyinEntry {
        ch: '\u{5417}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{541B}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{541D}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{541E}',
        readings: &["tun"],
    },
    CharPinyinEntry {
        ch: '\u{541F}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{5420}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{5421}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5423}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{5426}',
        readings: &["fou", "pi"],
    },
    CharPinyinEntry {
        ch: '\u{5427}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{5428}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{5429}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{542B}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{542C}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{542D}',
        readings: &["hang", "keng"],
    },
    CharPinyinEntry {
        ch: '\u{542E}',
        readings: &["shun"],
    },
    CharPinyinEntry {
        ch: '\u{542F}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{5431}',
        readings: &["zhi", "zi"],
    },
    CharPinyinEntry {
        ch: '\u{5432}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{5434}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5435}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{5438}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5439}',
        readings: &["chui"],
    },
    CharPinyinEntry {
        ch: '\u{543B}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{543C}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{543D}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{543E}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5440}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{5443}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{5446}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{5447}',
        readings: &["men"],
    },
    CharPinyinEntry {
        ch: '\u{5448}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{544A}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{544B}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5450}',
        readings: &["na"],
    },
    CharPinyinEntry {
        ch: '\u{5453}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5454}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{5455}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{5456}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{5457}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{5458}',
        readings: &["yuan", "yun"],
    },
    CharPinyinEntry {
        ch: '\u{5459}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{545B}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{545C}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5462}',
        readings: &["ne", "ni"],
    },
    CharPinyinEntry {
        ch: '\u{5463}',
        readings: &["m"],
    },
    CharPinyinEntry {
        ch: '\u{5464}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{5466}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{5468}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{5471}',
        readings: &["gu", "gua"],
    },
    CharPinyinEntry {
        ch: '\u{5472}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{5473}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5475}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{5476}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{5477}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{5478}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{547B}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{547C}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{547D}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{5480}',
        readings: &["ju", "zui"],
    },
    CharPinyinEntry {
        ch: '\u{5482}',
        readings: &["za"],
    },
    CharPinyinEntry {
        ch: '\u{5484}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{5486}',
        readings: &["pao"],
    },
    CharPinyinEntry {
        ch: '\u{5487}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5489}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{548B}',
        readings: &["za", "ze", "zha"],
    },
    CharPinyinEntry {
        ch: '\u{548C}',
        readings: &["he", "hu", "huo"],
    },
    CharPinyinEntry {
        ch: '\u{548D}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{548E}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{548F}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{5490}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5492}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{5494}',
        readings: &["ka"],
    },
    CharPinyinEntry {
        ch: '\u{5495}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{5496}',
        readings: &["ga", "ka"],
    },
    CharPinyinEntry {
        ch: '\u{5499}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{549A}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{549B}',
        readings: &["ning"],
    },
    CharPinyinEntry {
        ch: '\u{549D}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{54A1}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{54A3}',
        readings: &["guang"],
    },
    CharPinyinEntry {
        ch: '\u{54A4}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{54A5}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{54A6}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{54A7}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{54A8}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{54A9}',
        readings: &["mie"],
    },
    CharPinyinEntry {
        ch: '\u{54AA}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{54AB}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{54AC}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{54AF}',
        readings: &["ge", "ka", "lo", "luo"],
    },
    CharPinyinEntry {
        ch: '\u{54B1}',
        readings: &["zan"],
    },
    CharPinyinEntry {
        ch: '\u{54B3}',
        readings: &["hai", "ke"],
    },
    CharPinyinEntry {
        ch: '\u{54B4}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{54B8}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{54BA}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{54BB}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{54BD}',
        readings: &["yan", "ye"],
    },
    CharPinyinEntry {
        ch: '\u{54BF}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{54C0}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{54C1}',
        readings: &["pin"],
    },
    CharPinyinEntry {
        ch: '\u{54C2}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{54C3}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{54C4}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{54C6}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{54C7}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{54C8}',
        readings: &["ha"],
    },
    CharPinyinEntry {
        ch: '\u{54C9}',
        readings: &["zai"],
    },
    CharPinyinEntry {
        ch: '\u{54CC}',
        readings: &["pai"],
    },
    CharPinyinEntry {
        ch: '\u{54CD}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{54CE}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{54CF}',
        readings: &["gen"],
    },
    CharPinyinEntry {
        ch: '\u{54D0}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{54D1}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{54D2}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{54D3}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{54D4}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{54D5}',
        readings: &["hui", "yue"],
    },
    CharPinyinEntry {
        ch: '\u{54D7}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{54D9}',
        readings: &["kuai"],
    },
    CharPinyinEntry {
        ch: '\u{54DA}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{54DD}',
        readings: &["nong"],
    },
    CharPinyinEntry {
        ch: '\u{54DE}',
        readings: &["mou"],
    },
    CharPinyinEntry {
        ch: '\u{54DF}',
        readings: &["yo"],
    },
    CharPinyinEntry {
        ch: '\u{54E2}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{54E5}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{54E6}',
        readings: &["e", "o"],
    },
    CharPinyinEntry {
        ch: '\u{54E7}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{54E8}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{54E9}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{54EA}',
        readings: &["na", "ne"],
    },
    CharPinyinEntry {
        ch: '\u{54ED}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{54EE}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{54F1}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{54F2}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{54F3}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{54FA}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{54FC}',
        readings: &["heng", "hng"],
    },
    CharPinyinEntry {
        ch: '\u{54FD}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{54FF}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{5501}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5506}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{5507}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{5509}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{550F}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5510}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{5511}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{5514}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{551B}',
        readings: &["mai"],
    },
    CharPinyinEntry {
        ch: '\u{551D}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{5520}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{5522}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{5523}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{5524}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{5527}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{552A}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{552C}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{552E}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{552F}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5530}',
        readings: &["shua"],
    },
    CharPinyinEntry {
        ch: '\u{5531}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{5533}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{5535}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{5537}',
        readings: &["yo"],
    },
    CharPinyinEntry {
        ch: '\u{553C}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{553E}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{553F}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{5541}',
        readings: &["zhao", "zhou"],
    },
    CharPinyinEntry {
        ch: '\u{5543}',
        readings: &["ken"],
    },
    CharPinyinEntry {
        ch: '\u{5544}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{5546}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{5549}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{554A}',
        readings: &["a"],
    },
    CharPinyinEntry {
        ch: '\u{5550}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{5555}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{5556}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{555C}',
        readings: &["chuai", "chuo"],
    },
    CharPinyinEntry {
        ch: '\u{5561}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{5564}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{5565}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{5566}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{5567}',
        readings: &["ze"],
    },
    CharPinyinEntry {
        ch: '\u{556A}',
        readings: &["pa"],
    },
    CharPinyinEntry {
        ch: '\u{556B}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{556C}',
        readings: &["se"],
    },
    CharPinyinEntry {
        ch: '\u{556D}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{556E}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{5570}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{5574}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{5575}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{5576}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{5577}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{5578}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{557B}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{557C}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{557E}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{5580}',
        readings: &["ka"],
    },
    CharPinyinEntry {
        ch: '\u{5581}',
        readings: &["yong", "yu"],
    },
    CharPinyinEntry {
        ch: '\u{5582}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5583}',
        readings: &["nan"],
    },
    CharPinyinEntry {
        ch: '\u{5584}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{5586}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{5587}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{5588}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{5589}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{558A}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{558B}',
        readings: &["die", "zha"],
    },
    CharPinyinEntry {
        ch: '\u{558F}',
        readings: &["nuo"],
    },
    CharPinyinEntry {
        ch: '\u{5591}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{5594}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{5598}',
        readings: &["chuan"],
    },
    CharPinyinEntry {
        ch: '\u{5599}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{559C}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{559D}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{559F}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{55A4}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{55A7}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{55B1}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{55B3}',
        readings: &["cha", "zha"],
    },
    CharPinyinEntry {
        ch: '\u{55B5}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{55B7}',
        readings: &["pen"],
    },
    CharPinyinEntry {
        ch: '\u{55B9}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{55BB}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{55BD}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{55BE}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{55C4}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{55C5}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{55C9}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{55CC}',
        readings: &["ai", "yi"],
    },
    CharPinyinEntry {
        ch: '\u{55CD}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{55D0}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{55D1}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{55D2}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{55D3}',
        readings: &["sang"],
    },
    CharPinyinEntry {
        ch: '\u{55D4}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{55D6}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{55DC}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{55DD}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{55DE}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{55DF}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{55E1}',
        readings: &["weng"],
    },
    CharPinyinEntry {
        ch: '\u{55E3}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{55E4}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{55E5}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{55E6}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{55E8}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{55EA}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{55EB}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{55EC}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{55EF}',
        readings: &["ng"],
    },
    CharPinyinEntry {
        ch: '\u{55F2}',
        readings: &["dia"],
    },
    CharPinyinEntry {
        ch: '\u{55F3}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{55F5}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{55F7}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{55FD}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{55FE}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{5600}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{5601}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{5608}',
        readings: &["cao"],
    },
    CharPinyinEntry {
        ch: '\u{5609}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{560C}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{560E}',
        readings: &["ga"],
    },
    CharPinyinEntry {
        ch: '\u{560F}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{5618}',
        readings: &["shi", "xu"],
    },
    CharPinyinEntry {
        ch: '\u{561A}',
        readings: &["de", "dei"],
    },
    CharPinyinEntry {
        ch: '\u{561B}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{561E}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{561F}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{5621}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{5623}',
        readings: &["beng"],
    },
    CharPinyinEntry {
        ch: '\u{5624}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{5627}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{562C}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{562D}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{5631}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{5632}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{5634}',
        readings: &["zui"],
    },
    CharPinyinEntry {
        ch: '\u{5636}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{5639}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{563B}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{563F}',
        readings: &["hei"],
    },
    CharPinyinEntry {
        ch: '\u{5640}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{5642}',
        readings: &["zun"],
    },
    CharPinyinEntry {
        ch: '\u{5647}',
        readings: &["chuang"],
    },
    CharPinyinEntry {
        ch: '\u{564C}',
        readings: &["ceng"],
    },
    CharPinyinEntry {
        ch: '\u{564D}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{564E}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{5654}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{5657}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{5658}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{5659}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{565C}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{5662}',
        readings: &["o"],
    },
    CharPinyinEntry {
        ch: '\u{5664}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{5668}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{5669}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{566A}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{566B}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{566C}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5671}',
        readings: &["xue"],
    },
    CharPinyinEntry {
        ch: '\u{5676}',
        readings: &["ga"],
    },
    CharPinyinEntry {
        ch: '\u{567B}',
        readings: &["sai"],
    },
    CharPinyinEntry {
        ch: '\u{567C}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{5684}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{5685}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{5686}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{568E}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{568F}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{5693}',
        readings: &["ca", "cha"],
    },
    CharPinyinEntry {
        ch: '\u{569A}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{56A3}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{56AD}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{56AF}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{56B7}',
        readings: &["rang"],
    },
    CharPinyinEntry {
        ch: '\u{56BC}',
        readings: &["jiao", "jue"],
    },
    CharPinyinEntry {
        ch: '\u{56CA}',
        readings: &["nang"],
    },
    CharPinyinEntry {
        ch: '\u{56D4}',
        readings: &["nang"],
    },
    CharPinyinEntry {
        ch: '\u{56DA}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{56DB}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{56DE}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{56DF}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{56E0}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{56E1}',
        readings: &["nan"],
    },
    CharPinyinEntry {
        ch: '\u{56E2}',
        readings: &["tuan"],
    },
    CharPinyinEntry {
        ch: '\u{56E4}',
        readings: &["dun", "tun"],
    },
    CharPinyinEntry {
        ch: '\u{56EB}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{56ED}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{56F0}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{56F1}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{56F4}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{56F5}',
        readings: &["lun"],
    },
    CharPinyinEntry {
        ch: '\u{56F7}',
        readings: &["qun"],
    },
    CharPinyinEntry {
        ch: '\u{56F9}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{56FA}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{56FD}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{56FE}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{56FF}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{5703}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{5704}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{5706}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{5708}',
        readings: &["juan", "quan"],
    },
    CharPinyinEntry {
        ch: '\u{5709}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{570A}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{570C}',
        readings: &["chui"],
    },
    CharPinyinEntry {
        ch: '\u{5710}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{5719}',
        readings: &["lve"],
    },
    CharPinyinEntry {
        ch: '\u{571C}',
        readings: &["huan", "yuan"],
    },
    CharPinyinEntry {
        ch: '\u{571F}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{5722}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{5723}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{5728}',
        readings: &["zai"],
    },
    CharPinyinEntry {
        ch: '\u{5729}',
        readings: &["wei", "xu"],
    },
    CharPinyinEntry {
        ch: '\u{572A}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{572B}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{572C}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{572D}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{572E}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{572F}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5730}',
        readings: &["de", "di"],
    },
    CharPinyinEntry {
        ch: '\u{5732}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{5733}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{5739}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{573A}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{573B}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{573E}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5740}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{5742}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{5747}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{5749}',
        readings: &["tun"],
    },
    CharPinyinEntry {
        ch: '\u{574A}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{574B}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{574C}',
        readings: &["ben"],
    },
    CharPinyinEntry {
        ch: '\u{574D}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{574E}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{574F}',
        readings: &["huai"],
    },
    CharPinyinEntry {
        ch: '\u{5750}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{5751}',
        readings: &["keng"],
    },
    CharPinyinEntry {
        ch: '\u{5752}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5757}',
        readings: &["kuai"],
    },
    CharPinyinEntry {
        ch: '\u{575A}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{575B}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{575C}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{575D}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{575E}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{575F}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{5760}',
        readings: &["zhui"],
    },
    CharPinyinEntry {
        ch: '\u{5761}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{5764}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{5765}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{5766}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{5768}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{5769}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{576A}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{576B}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{576C}',
        readings: &["gua", "wa"],
    },
    CharPinyinEntry {
        ch: '\u{576D}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{576F}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{5770}',
        readings: &["jiong"],
    },
    CharPinyinEntry {
        ch: '\u{5773}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{5777}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{577B}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{577C}',
        readings: &["che"],
    },
    CharPinyinEntry {
        ch: '\u{577D}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{5782}',
        readings: &["chui"],
    },
    CharPinyinEntry {
        ch: '\u{5783}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{5784}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{5786}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{5788}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{578B}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{578C}',
        readings: &["dong", "tong"],
    },
    CharPinyinEntry {
        ch: '\u{578D}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{578E}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{578F}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{5792}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{5793}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{5795}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{5799}',
        readings: &["guang"],
    },
    CharPinyinEntry {
        ch: '\u{579A}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{579B}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{579E}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{579F}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{57A0}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{57A1}',
        readings: &["fa"],
    },
    CharPinyinEntry {
        ch: '\u{57A2}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{57A3}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{57A4}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{57A6}',
        readings: &["ken"],
    },
    CharPinyinEntry {
        ch: '\u{57A7}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{57A9}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{57AB}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{57AD}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{57AE}',
        readings: &["kua"],
    },
    CharPinyinEntry {
        ch: '\u{57AF}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{57B1}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{57B2}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{57B4}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{57B5}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{57B8}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{57BA}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{57BE}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{57BF}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{57C2}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{57C3}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{57C6}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{57C7}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{57CB}',
        readings: &["mai", "man"],
    },
    CharPinyinEntry {
        ch: '\u{57CC}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{57CE}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{57CF}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{57D2}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{57D4}',
        readings: &["bu", "pu"],
    },
    CharPinyinEntry {
        ch: '\u{57D5}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{57D7}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{57D8}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{57D9}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{57DA}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{57DD}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{57DF}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{57E0}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{57E4}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{57EA}',
        readings: &["kong"],
    },
    CharPinyinEntry {
        ch: '\u{57EB}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{57ED}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{57EF}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{57F4}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{57F5}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{57F8}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{57F9}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{57FA}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{57FC}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{57FD}',
        readings: &["sao"],
    },
    CharPinyinEntry {
        ch: '\u{5802}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{5803}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{5806}',
        readings: &["dui"],
    },
    CharPinyinEntry {
        ch: '\u{5807}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{5809}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{580B}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{580C}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{580D}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{580E}',
        readings: &["leng"],
    },
    CharPinyinEntry {
        ch: '\u{5810}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{5811}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{5815}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{5819}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{581E}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{5820}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{5821}',
        readings: &["bao", "bu", "pu"],
    },
    CharPinyinEntry {
        ch: '\u{5824}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{5827}',
        readings: &["ruan"],
    },
    CharPinyinEntry {
        ch: '\u{5828}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{582A}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{5830}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5832}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5835}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{583C}',
        readings: &["heng"],
    },
    CharPinyinEntry {
        ch: '\u{583D}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{583E}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{5844}',
        readings: &["leng"],
    },
    CharPinyinEntry {
        ch: '\u{5845}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{5846}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{584C}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{584D}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{5851}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{5854}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{5858}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{585D}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{585E}',
        readings: &["sai", "se"],
    },
    CharPinyinEntry {
        ch: '\u{5865}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{586B}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{586C}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{5871}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{587E}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{5880}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{5881}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{5883}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{5885}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{5888}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{5889}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{5890}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{5892}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{5893}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{5895}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5898}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{5899}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{589A}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{589E}',
        readings: &["zeng"],
    },
    CharPinyinEntry {
        ch: '\u{589F}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{58A1}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{58A3}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{58A6}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{58A8}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{58A9}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{58BC}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{58C1}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{58C5}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{58D1}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{58D5}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{58E4}',
        readings: &["rang"],
    },
    CharPinyinEntry {
        ch: '\u{58EB}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{58EC}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{58EE}',
        readings: &["zhuang"],
    },
    CharPinyinEntry {
        ch: '\u{58F0}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{58F3}',
        readings: &["ke", "qiao"],
    },
    CharPinyinEntry {
        ch: '\u{58F6}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{58F8}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{58F9}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5904}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{5907}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{590D}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{590F}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{5910}',
        readings: &["xiong"],
    },
    CharPinyinEntry {
        ch: '\u{5914}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{5915}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5916}',
        readings: &["wai"],
    },
    CharPinyinEntry {
        ch: '\u{5919}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{591A}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{591C}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{591F}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{5924}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{5925}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{5927}',
        readings: &["da", "dai"],
    },
    CharPinyinEntry {
        ch: '\u{5929}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{592A}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{592B}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{592C}',
        readings: &["guai"],
    },
    CharPinyinEntry {
        ch: '\u{592D}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{592E}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{592F}',
        readings: &["hang"],
    },
    CharPinyinEntry {
        ch: '\u{5931}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5934}',
        readings: &["tou"],
    },
    CharPinyinEntry {
        ch: '\u{5937}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5938}',
        readings: &["kua"],
    },
    CharPinyinEntry {
        ch: '\u{5939}',
        readings: &["ga", "jia"],
    },
    CharPinyinEntry {
        ch: '\u{593A}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{593C}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{5941}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{5942}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{5944}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5947}',
        readings: &["ji", "qi"],
    },
    CharPinyinEntry {
        ch: '\u{5948}',
        readings: &["nai"],
    },
    CharPinyinEntry {
        ch: '\u{5949}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{594B}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{594E}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{594F}',
        readings: &["zou"],
    },
    CharPinyinEntry {
        ch: '\u{5951}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{5953}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{5954}',
        readings: &["ben"],
    },
    CharPinyinEntry {
        ch: '\u{5955}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5956}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{5957}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{5958}',
        readings: &["zang", "zhuang"],
    },
    CharPinyinEntry {
        ch: '\u{595A}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5960}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{5961}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{5962}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{5965}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{596D}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5973}',
        readings: &["nv"],
    },
    CharPinyinEntry {
        ch: '\u{5974}',
        readings: &["nu"],
    },
    CharPinyinEntry {
        ch: '\u{5976}',
        readings: &["nai"],
    },
    CharPinyinEntry {
        ch: '\u{5978}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{5979}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{597D}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{5981}',
        readings: &["shuo"],
    },
    CharPinyinEntry {
        ch: '\u{5982}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{5983}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{5984}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{5986}',
        readings: &["zhuang"],
    },
    CharPinyinEntry {
        ch: '\u{5987}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5988}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{598A}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{598D}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5992}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{5993}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5996}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{5997}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{5998}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{5999}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{599E}',
        readings: &["niu"],
    },
    CharPinyinEntry {
        ch: '\u{59A3}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{59A4}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{59A5}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{59A7}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{59A8}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{59A9}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{59AA}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{59AB}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{59AD}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{59AE}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{59AF}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{59B2}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{59B9}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{59BB}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{59BE}',
        readings: &["qie"],
    },
    CharPinyinEntry {
        ch: '\u{59C6}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{59C8}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{59CA}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{59CB}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{59D0}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{59D1}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{59D2}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{59D3}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{59D4}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{59D7}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{59D8}',
        readings: &["pin"],
    },
    CharPinyinEntry {
        ch: '\u{59DA}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{59DC}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{59DD}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{59DE}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{59E3}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{59E4}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{59E5}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{59E8}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{59EC}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{59EE}',
        readings: &["heng"],
    },
    CharPinyinEntry {
        ch: '\u{59F1}',
        readings: &["kua"],
    },
    CharPinyinEntry {
        ch: '\u{59F6}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{59F9}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{59FB}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{59FD}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{59FF}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{5A00}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{5A01}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5A03}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{5A04}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{5A05}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{5A06}',
        readings: &["rao"],
    },
    CharPinyinEntry {
        ch: '\u{5A07}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{5A08}',
        readings: &["luan"],
    },
    CharPinyinEntry {
        ch: '\u{5A09}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{5A0C}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{5A11}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{5A13}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5A18}',
        readings: &["niang"],
    },
    CharPinyinEntry {
        ch: '\u{5A1C}',
        readings: &["na", "nuo"],
    },
    CharPinyinEntry {
        ch: '\u{5A1F}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{5A20}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{5A23}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{5A25}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{5A29}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{5A31}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{5A32}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{5A34}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{5A35}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{5A36}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{5A3C}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{5A40}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{5A46}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{5A49}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{5A4A}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{5A4C}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{5A4D}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{5A55}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{5A58}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{5A5A}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{5A5E}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{5A60}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{5A62}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5A64}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{5A67}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{5A6A}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{5A6B}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{5A73}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{5A74}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{5A75}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{5A76}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{5A77}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{5A7A}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5A7B}',
        readings: &["nan"],
    },
    CharPinyinEntry {
        ch: '\u{5A7C}',
        readings: &["ruo"],
    },
    CharPinyinEntry {
        ch: '\u{5A7F}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{5A82}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{5A84}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{5A86}',
        readings: &["ruan"],
    },
    CharPinyinEntry {
        ch: '\u{5A92}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{5A93}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{5A96}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{5A9A}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{5A9B}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{5A9E}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5AAA}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{5AAD}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{5AB1}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{5AB2}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{5AB3}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5AB5}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{5AB8}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{5ABE}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{5AC1}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{5AC2}',
        readings: &["sao"],
    },
    CharPinyinEntry {
        ch: '\u{5AC4}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{5AC9}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5ACC}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{5AD2}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{5AD4}',
        readings: &["pin"],
    },
    CharPinyinEntry {
        ch: '\u{5AD5}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5AD6}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{5AD8}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{5ADA}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{5ADC}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{5AE0}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{5AE1}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{5AE3}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5AE6}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{5AE9}',
        readings: &["nen"],
    },
    CharPinyinEntry {
        ch: '\u{5AEA}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{5AEB}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{5AED}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{5AF1}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{5AFD}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{5B09}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5B16}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5B17}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{5B1B}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{5B25}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{5B2C}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{5B34}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{5B37}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{5B3F}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5B40}',
        readings: &["shuang"],
    },
    CharPinyinEntry {
        ch: '\u{5B45}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{5B50}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{5B51}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{5B53}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{5B54}',
        readings: &["kong"],
    },
    CharPinyinEntry {
        ch: '\u{5B55}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{5B56}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{5B57}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{5B58}',
        readings: &["cun"],
    },
    CharPinyinEntry {
        ch: '\u{5B59}',
        readings: &["sun"],
    },
    CharPinyinEntry {
        ch: '\u{5B5A}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5B5B}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{5B5C}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{5B5D}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{5B5F}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{5B62}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{5B63}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5B64}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{5B65}',
        readings: &["nu"],
    },
    CharPinyinEntry {
        ch: '\u{5B66}',
        readings: &["xue"],
    },
    CharPinyinEntry {
        ch: '\u{5B69}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{5B6A}',
        readings: &["luan"],
    },
    CharPinyinEntry {
        ch: '\u{5B6C}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{5B70}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{5B71}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{5B73}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{5B75}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5B7A}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{5B7D}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{5B81}',
        readings: &["ning"],
    },
    CharPinyinEntry {
        ch: '\u{5B83}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{5B84}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{5B85}',
        readings: &["zhai"],
    },
    CharPinyinEntry {
        ch: '\u{5B87}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{5B88}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{5B89}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{5B8B}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{5B8C}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{5B8F}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{5B93}',
        readings: &["fu", "mi"],
    },
    CharPinyinEntry {
        ch: '\u{5B95}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{5B97}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{5B98}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{5B99}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{5B9A}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{5B9B}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{5B9C}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5B9D}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{5B9E}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5BA0}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{5BA1}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{5BA2}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{5BA3}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{5BA4}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5BA5}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{5BA6}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{5BA7}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5BAA}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{5BAB}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{5BAC}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{5BB0}',
        readings: &["zai"],
    },
    CharPinyinEntry {
        ch: '\u{5BB3}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{5BB4}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5BB5}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{5BB6}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{5BB8}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{5BB9}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{5BBD}',
        readings: &["kuan"],
    },
    CharPinyinEntry {
        ch: '\u{5BBE}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{5BBF}',
        readings: &["su", "xiu"],
    },
    CharPinyinEntry {
        ch: '\u{5BC1}',
        readings: &["zan"],
    },
    CharPinyinEntry {
        ch: '\u{5BC2}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5BC4}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5BC5}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{5BC6}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{5BC7}',
        readings: &["kou"],
    },
    CharPinyinEntry {
        ch: '\u{5BCC}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5BD0}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{5BD2}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{5BD3}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{5BDD}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{5BDE}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{5BDF}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{5BE1}',
        readings: &["gua"],
    },
    CharPinyinEntry {
        ch: '\u{5BE4}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5BE5}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{5BE8}',
        readings: &["zhai"],
    },
    CharPinyinEntry {
        ch: '\u{5BEE}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{5BF0}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{5BF8}',
        readings: &["cun"],
    },
    CharPinyinEntry {
        ch: '\u{5BF9}',
        readings: &["dui"],
    },
    CharPinyinEntry {
        ch: '\u{5BFA}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{5BFB}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{5BFC}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{5BFF}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{5C01}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{5C04}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{5C06}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{5C09}',
        readings: &["wei", "yu"],
    },
    CharPinyinEntry {
        ch: '\u{5C0A}',
        readings: &["zun"],
    },
    CharPinyinEntry {
        ch: '\u{5C0F}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{5C11}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{5C14}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{5C15}',
        readings: &["ga"],
    },
    CharPinyinEntry {
        ch: '\u{5C16}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{5C18}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{5C1A}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{5C1C}',
        readings: &["ga"],
    },
    CharPinyinEntry {
        ch: '\u{5C1D}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{5C22}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{5C24}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{5C25}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{5C27}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{5C28}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{5C2A}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{5C2C}',
        readings: &["ga"],
    },
    CharPinyinEntry {
        ch: '\u{5C31}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{5C34}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{5C38}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5C39}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{5C3A}',
        readings: &["che", "chi"],
    },
    CharPinyinEntry {
        ch: '\u{5C3B}',
        readings: &["kao"],
    },
    CharPinyinEntry {
        ch: '\u{5C3C}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{5C3D}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{5C3E}',
        readings: &["wei", "yi"],
    },
    CharPinyinEntry {
        ch: '\u{5C3F}',
        readings: &["niao", "sui"],
    },
    CharPinyinEntry {
        ch: '\u{5C40}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{5C41}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{5C42}',
        readings: &["ceng"],
    },
    CharPinyinEntry {
        ch: '\u{5C43}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5C45}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{5C48}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{5C49}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{5C4A}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{5C4B}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5C4E}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5C4F}',
        readings: &["bing", "ping"],
    },
    CharPinyinEntry {
        ch: '\u{5C50}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5C51}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{5C55}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{5C59}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{5C5E}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{5C60}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{5C61}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{5C63}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5C65}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{5C66}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{5C6F}',
        readings: &["tun"],
    },
    CharPinyinEntry {
        ch: '\u{5C71}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{5C79}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5C7A}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{5C7C}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5C7E}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{5C7F}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{5C81}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{5C82}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{5C88}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{5C8A}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{5C8C}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5C8D}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{5C90}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{5C91}',
        readings: &["cen"],
    },
    CharPinyinEntry {
        ch: '\u{5C94}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{5C96}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{5C97}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{5C98}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{5C99}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{5C9A}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{5C9B}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{5C9C}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{5C9E}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{5CA0}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{5CA2}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{5CA3}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{5CA8}',
        readings: &["ju", "qu"],
    },
    CharPinyinEntry {
        ch: '\u{5CA9}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5CAB}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{5CAC}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{5CAD}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{5CB1}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{5CB3}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{5CB5}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{5CB7}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{5CB8}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{5CBD}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{5CBF}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{5CC1}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{5CC2}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{5CC3}',
        readings: &["xue"],
    },
    CharPinyinEntry {
        ch: '\u{5CC4}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5CCB}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{5CD2}',
        readings: &["dong", "tong"],
    },
    CharPinyinEntry {
        ch: '\u{5CD7}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5CD8}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{5CD9}',
        readings: &["shi", "zhi"],
    },
    CharPinyinEntry {
        ch: '\u{5CDB}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{5CE1}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{5CE3}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{5CE4}',
        readings: &["jiao", "qiao"],
    },
    CharPinyinEntry {
        ch: '\u{5CE5}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{5CE6}',
        readings: &["luan"],
    },
    CharPinyinEntry {
        ch: '\u{5CE7}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{5CE8}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{5CEA}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{5CED}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{5CF0}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{5CF1}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{5CFB}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{5CFF}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5D00}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{5D01}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{5D02}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{5D03}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{5D04}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{5D06}',
        readings: &["kong"],
    },
    CharPinyinEntry {
        ch: '\u{5D07}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{5D0C}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{5D0E}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{5D12}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{5D14}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{5D16}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{5D1A}',
        readings: &["leng"],
    },
    CharPinyinEntry {
        ch: '\u{5D1B}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{5D1E}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{5D1F}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{5D21}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{5D24}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{5D26}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5D27}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{5D29}',
        readings: &["beng"],
    },
    CharPinyinEntry {
        ch: '\u{5D2D}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{5D2E}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{5D34}',
        readings: &["wai"],
    },
    CharPinyinEntry {
        ch: '\u{5D36}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{5D3D}',
        readings: &["zai"],
    },
    CharPinyinEntry {
        ch: '\u{5D3E}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{5D3F}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{5D41}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{5D45}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{5D47}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5D4A}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{5D4B}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{5D4C}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{5D4E}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{5D56}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{5D58}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{5D5A}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{5D5B}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{5D5D}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{5D69}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{5D6B}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{5D6C}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5D6F}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{5D72}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{5D74}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5D82}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{5D85}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{5D8D}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5D92}',
        readings: &["ceng"],
    },
    CharPinyinEntry {
        ch: '\u{5D93}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{5D99}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{5D9D}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{5D9F}',
        readings: &["zun"],
    },
    CharPinyinEntry {
        ch: '\u{5DA6}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{5DB2}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5DB7}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5DC5}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{5DC7}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5DC9}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{5DCD}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5DDD}',
        readings: &["chuan"],
    },
    CharPinyinEntry {
        ch: '\u{5DDE}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{5DE1}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{5DE2}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{5DE5}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{5DE6}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{5DE7}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{5DE8}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{5DE9}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{5DEB}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5DEE}',
        readings: &["cha", "chai", "ci"],
    },
    CharPinyinEntry {
        ch: '\u{5DEF}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{5DF1}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5DF2}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5DF3}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{5DF4}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{5DF7}',
        readings: &["hang", "xiang"],
    },
    CharPinyinEntry {
        ch: '\u{5DFD}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{5DFE}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{5E01}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5E02}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5E03}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{5E05}',
        readings: &["shuai"],
    },
    CharPinyinEntry {
        ch: '\u{5E06}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{5E08}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5E0C}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5E0F}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5E10}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{5E11}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{5E14}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{5E15}',
        readings: &["pa"],
    },
    CharPinyinEntry {
        ch: '\u{5E16}',
        readings: &["tie"],
    },
    CharPinyinEntry {
        ch: '\u{5E18}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{5E19}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{5E1A}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{5E1B}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{5E1C}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{5E1D}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{5E21}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{5E26}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{5E27}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{5E28}',
        readings: &["shui"],
    },
    CharPinyinEntry {
        ch: '\u{5E2D}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5E2E}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{5E31}',
        readings: &["chou", "dao"],
    },
    CharPinyinEntry {
        ch: '\u{5E37}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5E38}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{5E3B}',
        readings: &["ze"],
    },
    CharPinyinEntry {
        ch: '\u{5E3C}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{5E3D}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{5E42}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{5E44}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{5E45}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5E4C}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{5E54}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{5E55}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{5E56}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{5E5B}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{5E5E}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5E61}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{5E62}',
        readings: &["chuang", "zhuang"],
    },
    CharPinyinEntry {
        ch: '\u{5E6A}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{5E72}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{5E73}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{5E74}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{5E76}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{5E78}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{5E7A}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{5E7B}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{5E7C}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{5E7D}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{5E7F}',
        readings: &["guang"],
    },
    CharPinyinEntry {
        ch: '\u{5E84}',
        readings: &["zhuang"],
    },
    CharPinyinEntry {
        ch: '\u{5E86}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{5E87}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5E8A}',
        readings: &["chuang"],
    },
    CharPinyinEntry {
        ch: '\u{5E8B}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{5E8F}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{5E90}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{5E91}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5E93}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{5E94}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{5E95}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{5E96}',
        readings: &["pao"],
    },
    CharPinyinEntry {
        ch: '\u{5E97}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{5E99}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{5E9A}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{5E9C}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5E9E}',
        readings: &["pang"],
    },
    CharPinyinEntry {
        ch: '\u{5E9F}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{5EA0}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{5EA4}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{5EA5}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{5EA6}',
        readings: &["du", "duo"],
    },
    CharPinyinEntry {
        ch: '\u{5EA7}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{5EAD}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{5EB1}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{5EB3}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5EB5}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{5EB6}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{5EB7}',
        readings: &["kang"],
    },
    CharPinyinEntry {
        ch: '\u{5EB8}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{5EB9}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{5EBC}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{5EBE}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{5EC6}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5EC9}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{5ECA}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{5ECB}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{5ED1}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{5ED2}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{5ED3}',
        readings: &["kuo"],
    },
    CharPinyinEntry {
        ch: '\u{5ED6}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{5ED9}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5EDB}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{5EE8}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{5EEA}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{5EF6}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5EF7}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{5EFA}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{5EFF}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{5F00}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{5F01}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{5F02}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5F03}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{5F04}',
        readings: &["long", "nong"],
    },
    CharPinyinEntry {
        ch: '\u{5F06}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{5F07}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5F08}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5F0A}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5F0B}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5F0F}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5F11}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{5F13}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{5F15}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{5F17}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{5F18}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{5F1B}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{5F1F}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{5F20}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{5F22}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{5F25}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{5F26}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{5F27}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{5F28}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{5F29}',
        readings: &["nu"],
    },
    CharPinyinEntry {
        ch: '\u{5F2D}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{5F2F}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{5F31}',
        readings: &["ruo"],
    },
    CharPinyinEntry {
        ch: '\u{5F36}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{5F38}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{5F39}',
        readings: &["dan", "tan"],
    },
    CharPinyinEntry {
        ch: '\u{5F3A}',
        readings: &["jiang", "qiang"],
    },
    CharPinyinEntry {
        ch: '\u{5F3C}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5F40}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{5F52}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{5F53}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{5F55}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{5F56}',
        readings: &["tuan"],
    },
    CharPinyinEntry {
        ch: '\u{5F57}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{5F58}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{5F5D}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5F5F}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{5F62}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{5F64}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{5F66}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{5F67}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{5F69}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{5F6A}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{5F6C}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{5F6D}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{5F70}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{5F71}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{5F73}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{5F77}',
        readings: &["pang"],
    },
    CharPinyinEntry {
        ch: '\u{5F79}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5F7B}',
        readings: &["che"],
    },
    CharPinyinEntry {
        ch: '\u{5F7C}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5F80}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{5F81}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{5F82}',
        readings: &["cu"],
    },
    CharPinyinEntry {
        ch: '\u{5F84}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{5F85}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{5F87}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{5F88}',
        readings: &["hen"],
    },
    CharPinyinEntry {
        ch: '\u{5F89}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{5F8A}',
        readings: &["huai"],
    },
    CharPinyinEntry {
        ch: '\u{5F8B}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{5F90}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{5F92}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{5F95}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{5F97}',
        readings: &["de", "dei"],
    },
    CharPinyinEntry {
        ch: '\u{5F98}',
        readings: &["pai"],
    },
    CharPinyinEntry {
        ch: '\u{5F99}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{5F9B}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5F9C}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{5FA1}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{5FA8}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{5FAA}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{5FAD}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{5FAE}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{5FB5}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{5FB7}',
        readings: &["de"],
    },
    CharPinyinEntry {
        ch: '\u{5FBC}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{5FBD}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{5FC3}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{5FC5}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{5FC6}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{5FC9}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{5FCC}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{5FCD}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{5FCF}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{5FD0}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{5FD1}',
        readings: &["te"],
    },
    CharPinyinEntry {
        ch: '\u{5FD2}',
        readings: &["tui"],
    },
    CharPinyinEntry {
        ch: '\u{5FD6}',
        readings: &["cun"],
    },
    CharPinyinEntry {
        ch: '\u{5FD7}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{5FD8}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{5FD9}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{5FDD}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{5FDE}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{5FE0}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{5FE1}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{5FE4}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{5FE7}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{5FEA}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{5FEB}',
        readings: &["kuai"],
    },
    CharPinyinEntry {
        ch: '\u{5FED}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{5FEE}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{5FF1}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{5FF3}',
        readings: &["tun"],
    },
    CharPinyinEntry {
        ch: '\u{5FF5}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{5FF8}',
        readings: &["niu"],
    },
    CharPinyinEntry {
        ch: '\u{5FFA}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{5FFB}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{5FFD}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{5FFE}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{5FFF}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{6000}',
        readings: &["huai"],
    },
    CharPinyinEntry {
        ch: '\u{6001}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{6002}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{6003}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{6004}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{6005}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{6006}',
        readings: &["chuang"],
    },
    CharPinyinEntry {
        ch: '\u{600A}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{600D}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{600E}',
        readings: &["zen"],
    },
    CharPinyinEntry {
        ch: '\u{600F}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{6012}',
        readings: &["nu"],
    },
    CharPinyinEntry {
        ch: '\u{6014}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{6015}',
        readings: &["pa"],
    },
    CharPinyinEntry {
        ch: '\u{6016}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{6019}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{601B}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{601C}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{601D}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{6020}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{6021}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6025}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6026}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{6027}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{6028}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{6029}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{602A}',
        readings: &["guai"],
    },
    CharPinyinEntry {
        ch: '\u{602B}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{602F}',
        readings: &["qie"],
    },
    CharPinyinEntry {
        ch: '\u{6035}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{603B}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{603C}',
        readings: &["dui"],
    },
    CharPinyinEntry {
        ch: '\u{603F}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6041}',
        readings: &["nen"],
    },
    CharPinyinEntry {
        ch: '\u{6042}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{6043}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{604B}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{604D}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{6050}',
        readings: &["kong"],
    },
    CharPinyinEntry {
        ch: '\u{6052}',
        readings: &["heng"],
    },
    CharPinyinEntry {
        ch: '\u{6053}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6054}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{6055}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6059}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{605A}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{605D}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{6062}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{6063}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{6064}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{6067}',
        readings: &["nv"],
    },
    CharPinyinEntry {
        ch: '\u{6068}',
        readings: &["hen"],
    },
    CharPinyinEntry {
        ch: '\u{6069}',
        readings: &["en"],
    },
    CharPinyinEntry {
        ch: '\u{606A}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{606B}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{606C}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{606D}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{606F}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6070}',
        readings: &["qia"],
    },
    CharPinyinEntry {
        ch: '\u{6073}',
        readings: &["ken"],
    },
    CharPinyinEntry {
        ch: '\u{6076}',
        readings: &["e", "wu"],
    },
    CharPinyinEntry {
        ch: '\u{6078}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{6079}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{607A}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{607B}',
        readings: &["ce"],
    },
    CharPinyinEntry {
        ch: '\u{607C}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{607D}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{607F}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{6083}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{6084}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{6086}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6088}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{6089}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{608C}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{608D}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{6092}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6094}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{6096}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{609A}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{609B}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{609D}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{609F}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{60A0}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{60A2}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{60A3}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{60A6}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{60A8}',
        readings: &["nin"],
    },
    CharPinyinEntry {
        ch: '\u{60AB}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{60AC}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{60AD}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{60AF}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{60B0}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{60B1}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{60B2}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{60B4}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{60B8}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{60BB}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{60BC}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{60C5}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{60C6}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{60C7}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{60CA}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{60CB}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{60CE}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{60D1}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{60D4}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{60D5}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{60D8}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{60D9}',
        readings: &["chuo"],
    },
    CharPinyinEntry {
        ch: '\u{60DA}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{60DB}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{60DC}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{60DD}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{60DF}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{60E0}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{60E6}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{60E7}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{60E8}',
        readings: &["can"],
    },
    CharPinyinEntry {
        ch: '\u{60E9}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{60EB}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{60EC}',
        readings: &["qie"],
    },
    CharPinyinEntry {
        ch: '\u{60ED}',
        readings: &["can"],
    },
    CharPinyinEntry {
        ch: '\u{60EE}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{60EF}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{60F0}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{60F3}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{60F4}',
        readings: &["zhui"],
    },
    CharPinyinEntry {
        ch: '\u{60F6}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{60F9}',
        readings: &["re"],
    },
    CharPinyinEntry {
        ch: '\u{60FA}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{6100}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{6101}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{6103}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{6106}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{6108}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6109}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{610D}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{610E}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{610F}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6110}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{6114}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{6115}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{611A}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{611F}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{6120}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{6123}',
        readings: &["leng"],
    },
    CharPinyinEntry {
        ch: '\u{6124}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{6126}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{6127}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{612B}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{612D}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{613F}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{6146}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{6148}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{614A}',
        readings: &["qian", "qie"],
    },
    CharPinyinEntry {
        ch: '\u{614C}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{614E}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{6151}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{6155}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{615D}',
        readings: &["te"],
    },
    CharPinyinEntry {
        ch: '\u{6162}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{6165}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{6167}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{6168}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{616C}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{616D}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{6170}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{6175}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{6177}',
        readings: &["kang"],
    },
    CharPinyinEntry {
        ch: '\u{618B}',
        readings: &["bie"],
    },
    CharPinyinEntry {
        ch: '\u{618E}',
        readings: &["zeng"],
    },
    CharPinyinEntry {
        ch: '\u{6194}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{6195}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{6199}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{61A7}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{61A8}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{61A9}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{61AC}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{61AD}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{61B7}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{61BA}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{61BE}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{61C2}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{61C8}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{61CA}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{61CB}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{61D1}',
        readings: &["men"],
    },
    CharPinyinEntry {
        ch: '\u{61D2}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{61D4}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{61E6}',
        readings: &["nuo"],
    },
    CharPinyinEntry {
        ch: '\u{61F5}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{61FF}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6206}',
        readings: &["gang", "zhuang"],
    },
    CharPinyinEntry {
        ch: '\u{6208}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{620A}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{620B}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{620C}',
        readings: &["qu", "xu"],
    },
    CharPinyinEntry {
        ch: '\u{620D}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{620E}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{620F}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6210}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{6211}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{6212}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{6215}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{6216}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{6217}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{6218}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{621A}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{621B}',
        readings: &["ga", "jia"],
    },
    CharPinyinEntry {
        ch: '\u{621F}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6221}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{6222}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6223}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{6224}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{6225}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{622A}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{622C}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{622D}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{622E}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{6233}',
        readings: &["chuo"],
    },
    CharPinyinEntry {
        ch: '\u{6234}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{6237}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{623D}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{623E}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{623F}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{6240}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{6241}',
        readings: &["bian", "pian"],
    },
    CharPinyinEntry {
        ch: '\u{6242}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{6243}',
        readings: &["jiong"],
    },
    CharPinyinEntry {
        ch: '\u{6245}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6246}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6247}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{6248}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{6249}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{624A}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{624B}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{624D}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{624E}',
        readings: &["za", "zha"],
    },
    CharPinyinEntry {
        ch: '\u{6251}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{6252}',
        readings: &["ba", "pa"],
    },
    CharPinyinEntry {
        ch: '\u{6253}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{6254}',
        readings: &["reng"],
    },
    CharPinyinEntry {
        ch: '\u{6258}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{625B}',
        readings: &["gang", "kang"],
    },
    CharPinyinEntry {
        ch: '\u{625E}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{6263}',
        readings: &["kou"],
    },
    CharPinyinEntry {
        ch: '\u{6266}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{6267}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6269}',
        readings: &["kuo"],
    },
    CharPinyinEntry {
        ch: '\u{626A}',
        readings: &["men"],
    },
    CharPinyinEntry {
        ch: '\u{626B}',
        readings: &["sao"],
    },
    CharPinyinEntry {
        ch: '\u{626C}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{626D}',
        readings: &["niu"],
    },
    CharPinyinEntry {
        ch: '\u{626E}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{626F}',
        readings: &["che"],
    },
    CharPinyinEntry {
        ch: '\u{6270}',
        readings: &["rao"],
    },
    CharPinyinEntry {
        ch: '\u{6273}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{6276}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{6279}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{627A}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{627C}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{627D}',
        readings: &["den"],
    },
    CharPinyinEntry {
        ch: '\u{627E}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{627F}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{6280}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6283}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{6284}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{6289}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{628A}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{6291}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6292}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6293}',
        readings: &["zhua"],
    },
    CharPinyinEntry {
        ch: '\u{6294}',
        readings: &["pou"],
    },
    CharPinyinEntry {
        ch: '\u{6295}',
        readings: &["tou"],
    },
    CharPinyinEntry {
        ch: '\u{6296}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{6297}',
        readings: &["kang"],
    },
    CharPinyinEntry {
        ch: '\u{6298}',
        readings: &["she", "zhe"],
    },
    CharPinyinEntry {
        ch: '\u{629A}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{629B}',
        readings: &["pao"],
    },
    CharPinyinEntry {
        ch: '\u{629F}',
        readings: &["tuan"],
    },
    CharPinyinEntry {
        ch: '\u{62A0}',
        readings: &["kou"],
    },
    CharPinyinEntry {
        ch: '\u{62A1}',
        readings: &["lun"],
    },
    CharPinyinEntry {
        ch: '\u{62A2}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{62A4}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{62A5}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{62A8}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{62AB}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{62AC}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{62B1}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{62B5}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{62B9}',
        readings: &["ma", "mo"],
    },
    CharPinyinEntry {
        ch: '\u{62BB}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{62BC}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{62BD}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{62BF}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{62C2}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{62C3}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{62C4}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{62C5}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{62C6}',
        readings: &["chai"],
    },
    CharPinyinEntry {
        ch: '\u{62C7}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{62C8}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{62C9}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{62CA}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{62CC}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{62CD}',
        readings: &["pai"],
    },
    CharPinyinEntry {
        ch: '\u{62CE}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{62D0}',
        readings: &["guai"],
    },
    CharPinyinEntry {
        ch: '\u{62D2}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{62D3}',
        readings: &["ta", "tuo"],
    },
    CharPinyinEntry {
        ch: '\u{62D4}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{62D6}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{62D7}',
        readings: &["ao", "niu"],
    },
    CharPinyinEntry {
        ch: '\u{62D8}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{62D9}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{62DB}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{62DC}',
        readings: &["bai"],
    },
    CharPinyinEntry {
        ch: '\u{62DF}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{62E2}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{62E3}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{62E4}',
        readings: &["qia"],
    },
    CharPinyinEntry {
        ch: '\u{62E5}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{62E6}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{62E7}',
        readings: &["ning"],
    },
    CharPinyinEntry {
        ch: '\u{62E8}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{62E9}',
        readings: &["ze", "zhai"],
    },
    CharPinyinEntry {
        ch: '\u{62EC}',
        readings: &["kuo"],
    },
    CharPinyinEntry {
        ch: '\u{62ED}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{62EE}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{62EF}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{62F1}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{62F3}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{62F4}',
        readings: &["shuan"],
    },
    CharPinyinEntry {
        ch: '\u{62F6}',
        readings: &["za", "zan"],
    },
    CharPinyinEntry {
        ch: '\u{62F7}',
        readings: &["kao"],
    },
    CharPinyinEntry {
        ch: '\u{62FC}',
        readings: &["pin"],
    },
    CharPinyinEntry {
        ch: '\u{62FD}',
        readings: &["zhuai"],
    },
    CharPinyinEntry {
        ch: '\u{62FE}',
        readings: &["she", "shi"],
    },
    CharPinyinEntry {
        ch: '\u{62FF}',
        readings: &["na"],
    },
    CharPinyinEntry {
        ch: '\u{6301}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{6302}',
        readings: &["gua"],
    },
    CharPinyinEntry {
        ch: '\u{6307}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6308}',
        readings: &["qie"],
    },
    CharPinyinEntry {
        ch: '\u{6309}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{630E}',
        readings: &["kua"],
    },
    CharPinyinEntry {
        ch: '\u{6311}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{6313}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{6316}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{631A}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{631B}',
        readings: &["luan"],
    },
    CharPinyinEntry {
        ch: '\u{631D}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{631E}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{631F}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{6320}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{6321}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{6323}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{6324}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6325}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{6326}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{6328}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{632A}',
        readings: &["nuo"],
    },
    CharPinyinEntry {
        ch: '\u{632B}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{632F}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{6332}',
        readings: &["sa", "suo"],
    },
    CharPinyinEntry {
        ch: '\u{6339}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{633A}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{633D}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{6342}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{6343}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{6345}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{6346}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{6349}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{634B}',
        readings: &["lv", "luo"],
    },
    CharPinyinEntry {
        ch: '\u{634C}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{634D}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{634E}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{634F}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{6350}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{6355}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{635E}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{635F}',
        readings: &["sun"],
    },
    CharPinyinEntry {
        ch: '\u{6361}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{6362}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{6363}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{6367}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{6369}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{636D}',
        readings: &["bai"],
    },
    CharPinyinEntry {
        ch: '\u{636E}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{636F}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{6376}',
        readings: &["chui"],
    },
    CharPinyinEntry {
        ch: '\u{6377}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{637A}',
        readings: &["na"],
    },
    CharPinyinEntry {
        ch: '\u{637B}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{637D}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{6380}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{6382}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{6387}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{6388}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{6389}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{638A}',
        readings: &["pou"],
    },
    CharPinyinEntry {
        ch: '\u{638C}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{638E}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{638F}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{6390}',
        readings: &["qia"],
    },
    CharPinyinEntry {
        ch: '\u{6392}',
        readings: &["pai"],
    },
    CharPinyinEntry {
        ch: '\u{6396}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{6398}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{639E}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{63A0}',
        readings: &["lve"],
    },
    CharPinyinEntry {
        ch: '\u{63A2}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{63A3}',
        readings: &["che"],
    },
    CharPinyinEntry {
        ch: '\u{63A5}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{63A7}',
        readings: &["kong"],
    },
    CharPinyinEntry {
        ch: '\u{63A8}',
        readings: &["tui"],
    },
    CharPinyinEntry {
        ch: '\u{63A9}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{63AA}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{63AC}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{63AD}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{63AE}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{63B0}',
        readings: &["bai"],
    },
    CharPinyinEntry {
        ch: '\u{63B3}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{63B4}',
        readings: &["guai"],
    },
    CharPinyinEntry {
        ch: '\u{63B7}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{63B8}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{63BA}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{63BC}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{63BE}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{63C4}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{63C6}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{63C9}',
        readings: &["rou"],
    },
    CharPinyinEntry {
        ch: '\u{63CD}',
        readings: &["zou"],
    },
    CharPinyinEntry {
        ch: '\u{63CF}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{63D0}',
        readings: &["di", "ti"],
    },
    CharPinyinEntry {
        ch: '\u{63D2}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{63D5}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{63D6}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{63E0}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{63E1}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{63E3}',
        readings: &["chuai"],
    },
    CharPinyinEntry {
        ch: '\u{63E9}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{63EA}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{63ED}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{63F3}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{63F4}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{63F6}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{63F8}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{63FD}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{63FF}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{6400}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{6401}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{6402}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{6405}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{640B}',
        readings: &["chuai"],
    },
    CharPinyinEntry {
        ch: '\u{640C}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{640F}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{6410}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{6412}',
        readings: &["bang", "peng"],
    },
    CharPinyinEntry {
        ch: '\u{6413}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{6414}',
        readings: &["sao"],
    },
    CharPinyinEntry {
        ch: '\u{641B}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{641C}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{641E}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{6420}',
        readings: &["shuo"],
    },
    CharPinyinEntry {
        ch: '\u{6421}',
        readings: &["sang"],
    },
    CharPinyinEntry {
        ch: '\u{6426}',
        readings: &["nuo"],
    },
    CharPinyinEntry {
        ch: '\u{642A}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{642C}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{642D}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{6434}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{643A}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{643D}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{6441}',
        readings: &["en"],
    },
    CharPinyinEntry {
        ch: '\u{6444}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{6445}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6446}',
        readings: &["bai"],
    },
    CharPinyinEntry {
        ch: '\u{6447}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{6448}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{644A}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{644F}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{6452}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{6454}',
        readings: &["shuai"],
    },
    CharPinyinEntry {
        ch: '\u{6458}',
        readings: &["zhai"],
    },
    CharPinyinEntry {
        ch: '\u{645B}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{645E}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{6467}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{6469}',
        readings: &["ma", "mo"],
    },
    CharPinyinEntry {
        ch: '\u{646D}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6474}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{6478}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{6479}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{647D}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{6482}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{6484}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{6485}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{6487}',
        readings: &["pie"],
    },
    CharPinyinEntry {
        ch: '\u{6491}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{6492}',
        readings: &["sa"],
    },
    CharPinyinEntry {
        ch: '\u{6495}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{6496}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{6499}',
        readings: &["zun"],
    },
    CharPinyinEntry {
        ch: '\u{649E}',
        readings: &["zhuang"],
    },
    CharPinyinEntry {
        ch: '\u{64A4}',
        readings: &["che"],
    },
    CharPinyinEntry {
        ch: '\u{64A9}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{64AC}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{64AD}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{64AE}',
        readings: &["cuo", "zuo"],
    },
    CharPinyinEntry {
        ch: '\u{64B0}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{64B5}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{64B7}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{64B8}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{64BA}',
        readings: &["cuan"],
    },
    CharPinyinEntry {
        ch: '\u{64BC}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{64C0}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{64C2}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{64C5}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{64CD}',
        readings: &["cao"],
    },
    CharPinyinEntry {
        ch: '\u{64CE}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{64D0}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{64D2}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{64D8}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{64DE}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{64E2}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{64E4}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{64E6}',
        readings: &["ca"],
    },
    CharPinyinEntry {
        ch: '\u{64FF}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{6500}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{6509}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{6512}',
        readings: &["cuan", "zan"],
    },
    CharPinyinEntry {
        ch: '\u{6518}',
        readings: &["rang"],
    },
    CharPinyinEntry {
        ch: '\u{6525}',
        readings: &["zuan"],
    },
    CharPinyinEntry {
        ch: '\u{652B}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{652E}',
        readings: &["nang"],
    },
    CharPinyinEntry {
        ch: '\u{652F}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6536}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{6538}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{6539}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{653B}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{653D}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{653E}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{653F}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{6545}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{6548}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{6549}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{654C}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{654F}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{6551}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{6554}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6555}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{6556}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{6559}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{655B}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{655D}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{655E}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{6562}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{6563}',
        readings: &["san"],
    },
    CharPinyinEntry {
        ch: '\u{6566}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{6569}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{656B}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{656C}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{6570}',
        readings: &["shu", "shuo"],
    },
    CharPinyinEntry {
        ch: '\u{6572}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{6574}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{6577}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{6587}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{658B}',
        readings: &["zhai"],
    },
    CharPinyinEntry {
        ch: '\u{658C}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{6590}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{6591}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{6593}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{6597}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{6599}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{659B}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{659C}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{659D}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{659F}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{65A0}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{65A1}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{65A4}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{65A5}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{65A7}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{65A9}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{65AB}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{65AD}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{65AF}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{65B0}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{65B6}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{65B9}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{65BC}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{65BD}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{65C1}',
        readings: &["pang"],
    },
    CharPinyinEntry {
        ch: '\u{65C3}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{65C4}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{65C5}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{65C6}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{65CB}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{65CC}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{65CE}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{65CF}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{65D0}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{65D2}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{65D6}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{65D7}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{65DE}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{65E0}',
        readings: &["mo", "wu"],
    },
    CharPinyinEntry {
        ch: '\u{65E2}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{65E5}',
        readings: &["ri"],
    },
    CharPinyinEntry {
        ch: '\u{65E6}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{65E7}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{65E8}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{65E9}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{65EC}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{65ED}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{65EE}',
        readings: &["ga"],
    },
    CharPinyinEntry {
        ch: '\u{65EF}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{65F0}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{65F1}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{65F4}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{65F5}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{65F6}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{65F7}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{65F8}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{65FA}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{65FB}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{65FF}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{6600}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{6602}',
        readings: &["ang"],
    },
    CharPinyinEntry {
        ch: '\u{6603}',
        readings: &["ze"],
    },
    CharPinyinEntry {
        ch: '\u{6604}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{6606}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{6607}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{6608}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{6609}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{660A}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{660C}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{660E}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{660F}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{6612}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{6613}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6614}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6615}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{6619}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{661D}',
        readings: &["zan"],
    },
    CharPinyinEntry {
        ch: '\u{661F}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{6620}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{6621}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{6623}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{6624}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{6625}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{6627}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{6628}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{662A}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{662B}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{662D}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{662F}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{6631}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6633}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{6634}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{6635}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{6636}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{663A}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{663C}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{663D}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{663E}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{6641}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{6643}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{6645}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{664A}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{664B}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{664C}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{664F}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{6650}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{6652}',
        readings: &["shai"],
    },
    CharPinyinEntry {
        ch: '\u{6653}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{6654}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{6655}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{6656}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{6657}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{6659}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{665A}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{665E}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{665F}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{6661}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{6662}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{6664}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{6666}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{6668}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{666A}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{666B}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{666E}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{666F}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{6670}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6671}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{6674}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{6676}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{6677}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{667A}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{667E}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{6682}',
        readings: &["zan"],
    },
    CharPinyinEntry {
        ch: '\u{6684}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{6685}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{6687}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{668C}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{6691}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6695}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{6696}',
        readings: &["nuan"],
    },
    CharPinyinEntry {
        ch: '\u{6697}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{669D}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{66A7}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{66A8}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{66AE}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{66B2}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{66B4}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{66B5}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{66B6}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{66B9}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{66BE}',
        readings: &["tun"],
    },
    CharPinyinEntry {
        ch: '\u{66BF}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{66C8}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{66CC}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{66D9}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{66DB}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{66DC}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{66DD}',
        readings: &["bao", "pu"],
    },
    CharPinyinEntry {
        ch: '\u{66E6}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{66E9}',
        readings: &["nang"],
    },
    CharPinyinEntry {
        ch: '\u{66F0}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{66F2}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{66F3}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{66F4}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{66F7}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{66F9}',
        readings: &["cao"],
    },
    CharPinyinEntry {
        ch: '\u{66FC}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{66FE}',
        readings: &["ceng", "zeng"],
    },
    CharPinyinEntry {
        ch: '\u{66FF}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{6700}',
        readings: &["zui"],
    },
    CharPinyinEntry {
        ch: '\u{6708}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{6709}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{670B}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{670D}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{670F}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{6710}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{6713}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{6714}',
        readings: &["shuo"],
    },
    CharPinyinEntry {
        ch: '\u{6715}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{6717}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{671B}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{671D}',
        readings: &["chao", "zhao"],
    },
    CharPinyinEntry {
        ch: '\u{671F}',
        readings: &["ji", "qi"],
    },
    CharPinyinEntry {
        ch: '\u{6726}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{6728}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{672A}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{672B}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{672C}',
        readings: &["ben"],
    },
    CharPinyinEntry {
        ch: '\u{672D}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{672F}',
        readings: &["shu", "zhu"],
    },
    CharPinyinEntry {
        ch: '\u{6731}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{6733}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{6734}',
        readings: &["piao", "po", "pu"],
    },
    CharPinyinEntry {
        ch: '\u{6735}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{6738}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{673A}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{673D}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{6740}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{6742}',
        readings: &["za"],
    },
    CharPinyinEntry {
        ch: '\u{6743}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{6744}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{6746}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{6748}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{6749}',
        readings: &["sha", "shan"],
    },
    CharPinyinEntry {
        ch: '\u{674C}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{674E}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{674F}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{6750}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{6751}',
        readings: &["cun"],
    },
    CharPinyinEntry {
        ch: '\u{6753}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{6755}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{6756}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{6759}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{675C}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{675E}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{675F}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6760}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{6761}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{6765}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{6767}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{6768}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{6769}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{676A}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{676D}',
        readings: &["hang"],
    },
    CharPinyinEntry {
        ch: '\u{676F}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{6770}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{6772}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{6773}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{6775}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{6777}',
        readings: &["pa"],
    },
    CharPinyinEntry {
        ch: '\u{677B}',
        readings: &["chou", "niu"],
    },
    CharPinyinEntry {
        ch: '\u{677C}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{677E}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{677F}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{6781}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6784}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{6785}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6787}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{6789}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{678B}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{678D}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6790}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6795}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{6797}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{6798}',
        readings: &["rui"],
    },
    CharPinyinEntry {
        ch: '\u{679A}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{679C}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{679D}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{679E}',
        readings: &["cong", "zong"],
    },
    CharPinyinEntry {
        ch: '\u{67A2}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{67A3}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{67A5}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{67A7}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{67A8}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{67AA}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{67AB}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{67AD}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{67AF}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{67B0}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{67B2}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{67B3}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{67B5}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{67B6}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{67B7}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{67B8}',
        readings: &["gou", "ju"],
    },
    CharPinyinEntry {
        ch: '\u{67B9}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{67C1}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{67C3}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{67C4}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{67C8}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{67CA}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{67CF}',
        readings: &["bai", "bo"],
    },
    CharPinyinEntry {
        ch: '\u{67D0}',
        readings: &["mou"],
    },
    CharPinyinEntry {
        ch: '\u{67D1}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{67D2}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{67D3}',
        readings: &["ran"],
    },
    CharPinyinEntry {
        ch: '\u{67D4}',
        readings: &["rou"],
    },
    CharPinyinEntry {
        ch: '\u{67D6}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{67D8}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{67D9}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{67DA}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{67DC}',
        readings: &["gui", "ju"],
    },
    CharPinyinEntry {
        ch: '\u{67DD}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{67DE}',
        readings: &["zha", "zuo"],
    },
    CharPinyinEntry {
        ch: '\u{67E0}',
        readings: &["ning"],
    },
    CharPinyinEntry {
        ch: '\u{67E2}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{67E5}',
        readings: &["cha", "zha"],
    },
    CharPinyinEntry {
        ch: '\u{67E9}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{67EC}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{67EF}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{67F0}',
        readings: &["nai"],
    },
    CharPinyinEntry {
        ch: '\u{67F1}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{67F3}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{67F4}',
        readings: &["chai"],
    },
    CharPinyinEntry {
        ch: '\u{67F7}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{67FD}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{67FF}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{6800}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6805}',
        readings: &["shan", "zha"],
    },
    CharPinyinEntry {
        ch: '\u{6807}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{6808}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{6809}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{680A}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{680B}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{680C}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{680E}',
        readings: &["li", "yue"],
    },
    CharPinyinEntry {
        ch: '\u{680F}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{6810}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{6811}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6812}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{6813}',
        readings: &["shuan"],
    },
    CharPinyinEntry {
        ch: '\u{6816}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6817}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{681D}',
        readings: &["gua"],
    },
    CharPinyinEntry {
        ch: '\u{681F}',
        readings: &["ben"],
    },
    CharPinyinEntry {
        ch: '\u{6821}',
        readings: &["jiao", "xiao"],
    },
    CharPinyinEntry {
        ch: '\u{6829}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{682A}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{6832}',
        readings: &["kao"],
    },
    CharPinyinEntry {
        ch: '\u{6833}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{6834}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{6837}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{6838}',
        readings: &["he", "hu"],
    },
    CharPinyinEntry {
        ch: '\u{6839}',
        readings: &["gen"],
    },
    CharPinyinEntry {
        ch: '\u{683B}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{683C}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{683D}',
        readings: &["zai"],
    },
    CharPinyinEntry {
        ch: '\u{683E}',
        readings: &["luan"],
    },
    CharPinyinEntry {
        ch: '\u{6840}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{6841}',
        readings: &["heng"],
    },
    CharPinyinEntry {
        ch: '\u{6842}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{6843}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{6844}',
        readings: &["guang"],
    },
    CharPinyinEntry {
        ch: '\u{6845}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{6846}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{6848}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{6849}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{684A}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{684C}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{684E}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6850}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{6851}',
        readings: &["sang"],
    },
    CharPinyinEntry {
        ch: '\u{6853}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{6854}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{6855}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{6860}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{6861}',
        readings: &["rao"],
    },
    CharPinyinEntry {
        ch: '\u{6862}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{6863}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{6864}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6865}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{6866}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{6867}',
        readings: &["gui", "hui"],
    },
    CharPinyinEntry {
        ch: '\u{6868}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{6869}',
        readings: &["zhuang"],
    },
    CharPinyinEntry {
        ch: '\u{686B}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{686F}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{6872}',
        readings: &["bo", "po"],
    },
    CharPinyinEntry {
        ch: '\u{6874}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{6876}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{6877}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{6879}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{6881}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{6883}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{6885}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{6886}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{688C}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{688F}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{6893}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{6897}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{68A0}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{68A2}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{68A3}',
        readings: &["chen", "qin"],
    },
    CharPinyinEntry {
        ch: '\u{68A6}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{68A7}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{68A8}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{68AD}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{68AF}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{68B0}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{68B3}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{68B4}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{68B5}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{68BC}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{68BD}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{68BE}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{68BF}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{68C0}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{68C1}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{68C2}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{68C9}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{68CB}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{68CD}',
        readings: &["gun"],
    },
    CharPinyinEntry {
        ch: '\u{68D0}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{68D2}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{68D3}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{68D5}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{68D8}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{68DA}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{68E0}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{68E3}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{68E4}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{68E8}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{68EA}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{68EB}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{68EC}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{68EE}',
        readings: &["sen"],
    },
    CharPinyinEntry {
        ch: '\u{68F0}',
        readings: &["chui"],
    },
    CharPinyinEntry {
        ch: '\u{68F1}',
        readings: &["leng", "ling"],
    },
    CharPinyinEntry {
        ch: '\u{68F5}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{68F9}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{68FA}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{68FB}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{68FC}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{68FD}',
        readings: &["chen", "shen"],
    },
    CharPinyinEntry {
        ch: '\u{6900}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{6901}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{6905}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6906}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{690B}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{690D}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{690E}',
        readings: &["zhui"],
    },
    CharPinyinEntry {
        ch: '\u{6910}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{6911}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{6912}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{6913}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{691F}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{6920}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{6924}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{692A}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{692D}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{6930}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{6934}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{6938}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6939}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{693D}',
        readings: &["chuan"],
    },
    CharPinyinEntry {
        ch: '\u{693F}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{6942}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{6952}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{6954}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{6957}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{6959}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{695A}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{695D}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{695E}',
        readings: &["leng"],
    },
    CharPinyinEntry {
        ch: '\u{6960}',
        readings: &["nan"],
    },
    CharPinyinEntry {
        ch: '\u{6963}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{6966}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{6969}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{696A}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{696B}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{696E}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{696F}',
        readings: &["shun"],
    },
    CharPinyinEntry {
        ch: '\u{6977}',
        readings: &["jie", "kai"],
    },
    CharPinyinEntry {
        ch: '\u{6978}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{6979}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{697C}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{6982}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{6983}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{6984}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{6985}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{6986}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6987}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{6988}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{6989}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{698D}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{6991}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{6994}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{6995}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{6996}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{699B}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{699C}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{69A7}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{69A8}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{69AB}',
        readings: &["sun"],
    },
    CharPinyinEntry {
        ch: '\u{69AD}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{69B0}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{69B1}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{69B4}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{69B7}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{69BB}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{69C1}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{69C3}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{69CA}',
        readings: &["shuo"],
    },
    CharPinyinEntry {
        ch: '\u{69CC}',
        readings: &["chui"],
    },
    CharPinyinEntry {
        ch: '\u{69CE}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{69D0}',
        readings: &["huai"],
    },
    CharPinyinEntry {
        ch: '\u{69D4}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{69DA}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{69DB}',
        readings: &["jian", "kan"],
    },
    CharPinyinEntry {
        ch: '\u{69DC}',
        readings: &["zui"],
    },
    CharPinyinEntry {
        ch: '\u{69DF}',
        readings: &["bin", "bing"],
    },
    CharPinyinEntry {
        ch: '\u{69E0}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{69ED}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{69F1}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{69F2}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{69FD}',
        readings: &["cao"],
    },
    CharPinyinEntry {
        ch: '\u{69FF}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{6A0A}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{6A17}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{6A18}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{6A1F}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{6A21}',
        readings: &["mo", "mu"],
    },
    CharPinyinEntry {
        ch: '\u{6A28}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6A2A}',
        readings: &["heng"],
    },
    CharPinyinEntry {
        ch: '\u{6A2F}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{6A31}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{6A35}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{6A3D}',
        readings: &["zun"],
    },
    CharPinyinEntry {
        ch: '\u{6A3E}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{6A44}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{6A47}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{6A50}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{6A51}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{6A58}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{6A59}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{6A5B}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{6A5E}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{6A61}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{6A65}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{6A66}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{6A71}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{6A79}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{6A7C}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{6A80}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{6A84}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6A8E}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{6A90}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{6A91}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{6A97}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{6A9E}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{6AA0}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{6AA9}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{6AAB}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{6AAC}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{6AC6}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{6B02}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{6B20}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{6B21}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{6B22}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{6B23}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{6B24}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6B27}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{6B32}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6B38}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{6B39}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6B3A}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6B3B}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{6B3E}',
        readings: &["kuan"],
    },
    CharPinyinEntry {
        ch: '\u{6B43}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{6B45}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{6B46}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{6B47}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{6B49}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{6B4C}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{6B59}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{6B62}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6B63}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{6B64}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{6B65}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{6B66}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{6B67}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6B6A}',
        readings: &["wai"],
    },
    CharPinyinEntry {
        ch: '\u{6B79}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{6B7B}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{6B7C}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{6B81}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{6B82}',
        readings: &["cu"],
    },
    CharPinyinEntry {
        ch: '\u{6B83}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{6B84}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{6B86}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{6B87}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{6B89}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{6B8A}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6B8B}',
        readings: &["can"],
    },
    CharPinyinEntry {
        ch: '\u{6B8D}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{6B92}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{6B93}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{6B96}',
        readings: &["shi", "zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6B9A}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{6B9B}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6BA1}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{6BA3}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{6BAA}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6BB3}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6BB4}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{6BB5}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{6BB7}',
        readings: &["yan", "yin"],
    },
    CharPinyinEntry {
        ch: '\u{6BBF}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{6BC1}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{6BC2}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{6BC5}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6BCB}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{6BCC}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{6BCD}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{6BCF}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{6BD0}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{6BD2}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{6BD3}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6BD4}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{6BD5}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{6BD6}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{6BD7}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{6BD9}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{6BDB}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{6BE1}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{6BEA}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{6BEB}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{6BEF}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{6BF3}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{6BF5}',
        readings: &["san"],
    },
    CharPinyinEntry {
        ch: '\u{6BF9}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6BFD}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{6C05}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{6C06}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{6C07}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{6C0D}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{6C0F}',
        readings: &["shi", "zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6C10}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{6C11}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{6C13}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{6C14}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6C15}',
        readings: &["pie"],
    },
    CharPinyinEntry {
        ch: '\u{6C16}',
        readings: &["nai"],
    },
    CharPinyinEntry {
        ch: '\u{6C18}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{6C19}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{6C1A}',
        readings: &["chuan"],
    },
    CharPinyinEntry {
        ch: '\u{6C1B}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{6C1F}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{6C21}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{6C22}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{6C24}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{6C26}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{6C27}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{6C28}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{6C29}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{6C2A}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{6C2E}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{6C2F}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{6C30}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{6C32}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{6C34}',
        readings: &["shui"],
    },
    CharPinyinEntry {
        ch: '\u{6C38}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{6C3E}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{6C3F}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{6C40}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{6C41}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6C42}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{6C46}',
        readings: &["cuan"],
    },
    CharPinyinEntry {
        ch: '\u{6C47}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{6C48}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{6C49}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{6C4A}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{6C4B}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{6C50}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6C54}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6C55}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{6C57}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{6C5B}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{6C5C}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{6C5D}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{6C5E}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{6C5F}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{6C60}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{6C61}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{6C64}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{6C67}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{6C68}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{6C69}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{6C6A}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{6C6B}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{6C6D}',
        readings: &["rui"],
    },
    CharPinyinEntry {
        ch: '\u{6C70}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{6C72}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6C74}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{6C76}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{6C79}',
        readings: &["xiong"],
    },
    CharPinyinEntry {
        ch: '\u{6C7D}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6C7E}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{6C81}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{6C82}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6C83}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{6C84}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{6C85}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{6C86}',
        readings: &["hang"],
    },
    CharPinyinEntry {
        ch: '\u{6C87}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{6C88}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{6C89}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{6C8C}',
        readings: &["dun", "zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{6C8F}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6C90}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{6C93}',
        readings: &["da", "ta"],
    },
    CharPinyinEntry {
        ch: '\u{6C94}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{6C98}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{6C99}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{6C9A}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6C9B}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{6C9F}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{6CA1}',
        readings: &["mei", "mo"],
    },
    CharPinyinEntry {
        ch: '\u{6CA3}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{6CA4}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{6CA5}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{6CA6}',
        readings: &["lun"],
    },
    CharPinyinEntry {
        ch: '\u{6CA7}',
        readings: &["cang"],
    },
    CharPinyinEntry {
        ch: '\u{6CA8}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{6CA9}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{6CAA}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{6CAB}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{6CAD}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6CAE}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{6CB1}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{6CB3}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{6CB8}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{6CB9}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{6CBA}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{6CBB}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6CBC}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{6CBD}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{6CBE}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{6CBF}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{6CC2}',
        readings: &["jiong"],
    },
    CharPinyinEntry {
        ch: '\u{6CC3}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{6CC4}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{6CC5}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{6CC7}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{6CC9}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{6CCA}',
        readings: &["bo", "po"],
    },
    CharPinyinEntry {
        ch: '\u{6CCC}',
        readings: &["bi", "mi"],
    },
    CharPinyinEntry {
        ch: '\u{6CD0}',
        readings: &["le"],
    },
    CharPinyinEntry {
        ch: '\u{6CD3}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{6CD4}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{6CD5}',
        readings: &["fa"],
    },
    CharPinyinEntry {
        ch: '\u{6CD6}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{6CD7}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{6CD9}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{6CDA}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{6CDB}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{6CDC}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6CDE}',
        readings: &["ning"],
    },
    CharPinyinEntry {
        ch: '\u{6CE0}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{6CE1}',
        readings: &["pao"],
    },
    CharPinyinEntry {
        ch: '\u{6CE2}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{6CE3}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6CE5}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{6CE8}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{6CEA}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{6CEB}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{6CEE}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{6CEF}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{6CF0}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{6CF1}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{6CF3}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{6CF5}',
        readings: &["beng"],
    },
    CharPinyinEntry {
        ch: '\u{6CF7}',
        readings: &["long", "shuang"],
    },
    CharPinyinEntry {
        ch: '\u{6CF8}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{6CFA}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{6CFB}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{6CFC}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{6CFD}',
        readings: &["ze"],
    },
    CharPinyinEntry {
        ch: '\u{6CFE}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{6D01}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{6D04}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{6D07}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{6D08}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{6D0B}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{6D0C}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{6D0E}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6D11}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{6D12}',
        readings: &["sa"],
    },
    CharPinyinEntry {
        ch: '\u{6D13}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6D17}',
        readings: &["xi", "xian"],
    },
    CharPinyinEntry {
        ch: '\u{6D18}',
        readings: &["kao"],
    },
    CharPinyinEntry {
        ch: '\u{6D19}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{6D1A}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{6D1B}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{6D1E}',
        readings: &["dong", "tong"],
    },
    CharPinyinEntry {
        ch: '\u{6D22}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6D23}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{6D25}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{6D27}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{6D28}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{6D2A}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{6D2B}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{6D2D}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{6D2E}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{6D31}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{6D32}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{6D33}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{6D34}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{6D35}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{6D38}',
        readings: &["guang"],
    },
    CharPinyinEntry {
        ch: '\u{6D39}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{6D3A}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{6D3B}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{6D3C}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{6D3D}',
        readings: &["qia"],
    },
    CharPinyinEntry {
        ch: '\u{6D3E}',
        readings: &["pai"],
    },
    CharPinyinEntry {
        ch: '\u{6D3F}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{6D41}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{6D43}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{6D45}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{6D46}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{6D47}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{6D48}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{6D49}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{6D4A}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{6D4B}',
        readings: &["ce"],
    },
    CharPinyinEntry {
        ch: '\u{6D4D}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{6D4E}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6D4F}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{6D50}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{6D51}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{6D52}',
        readings: &["hu", "xu"],
    },
    CharPinyinEntry {
        ch: '\u{6D53}',
        readings: &["nong"],
    },
    CharPinyinEntry {
        ch: '\u{6D54}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{6D55}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{6D59}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{6D5A}',
        readings: &["jun", "xun"],
    },
    CharPinyinEntry {
        ch: '\u{6D5B}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{6D5C}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{6D5E}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{6D5F}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{6D60}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6D61}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{6D63}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{6D65}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6D66}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{6D69}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{6D6A}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{6D6C}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{6D6D}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{6D6E}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{6D6F}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{6D70}',
        readings: &["li", "lian"],
    },
    CharPinyinEntry {
        ch: '\u{6D72}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{6D74}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6D77}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{6D78}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{6D7C}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{6D82}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{6D84}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{6D85}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{6D88}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{6D89}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{6D8C}',
        readings: &["chong", "yong"],
    },
    CharPinyinEntry {
        ch: '\u{6D8D}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{6D8E}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{6D90}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{6D91}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{6D93}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{6D94}',
        readings: &["cen"],
    },
    CharPinyinEntry {
        ch: '\u{6D95}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{6D98}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{6D9B}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{6D9D}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{6D9E}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{6D9F}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{6DA0}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{6DA1}',
        readings: &["guo", "wo"],
    },
    CharPinyinEntry {
        ch: '\u{6DA2}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{6DA3}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{6DA4}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{6DA6}',
        readings: &["run"],
    },
    CharPinyinEntry {
        ch: '\u{6DA7}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{6DA8}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{6DA9}',
        readings: &["se"],
    },
    CharPinyinEntry {
        ch: '\u{6DAA}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{6DAB}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{6DAE}',
        readings: &["shuan"],
    },
    CharPinyinEntry {
        ch: '\u{6DAF}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{6DB2}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{6DB4}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{6DB5}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{6DB8}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{6DBF}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{6DC0}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{6DC4}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{6DC5}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6DC6}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{6DC7}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6DCB}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{6DCC}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{6DCF}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{6DD1}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6DD6}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{6DD8}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{6DD9}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{6DDC}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{6DDD}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{6DDE}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{6DDF}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{6DE0}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{6DE1}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{6DE4}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6DE6}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{6DEB}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{6DEC}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{6DEE}',
        readings: &["huai"],
    },
    CharPinyinEntry {
        ch: '\u{6DEF}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6DF1}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{6DF3}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{6DF4}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{6DF7}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{6DF9}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{6DFB}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{6DFC}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{6E05}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{6E0A}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{6E0C}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{6E0D}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{6E0E}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{6E10}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{6E11}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{6E14}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6E17}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{6E1A}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{6E1D}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6E1F}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{6E20}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{6E21}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{6E23}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{6E24}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{6E25}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{6E29}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{6E2B}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{6E2D}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{6E2F}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{6E30}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{6E32}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{6E34}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{6E38}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{6E3A}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{6E3C}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{6E43}',
        readings: &["pai"],
    },
    CharPinyinEntry {
        ch: '\u{6E44}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{6E49}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{6E4D}',
        readings: &["tuan"],
    },
    CharPinyinEntry {
        ch: '\u{6E4E}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{6E51}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{6E53}',
        readings: &["pen"],
    },
    CharPinyinEntry {
        ch: '\u{6E54}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{6E56}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{6E58}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{6E5B}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{6E5C}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{6E5D}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{6E5F}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{6E63}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{6E6B}',
        readings: &["jiao", "qiu"],
    },
    CharPinyinEntry {
        ch: '\u{6E6E}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{6E72}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{6E74}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{6E7E}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{6E7F}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{6E81}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{6E83}',
        readings: &["hui", "kui"],
    },
    CharPinyinEntry {
        ch: '\u{6E85}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{6E86}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{6E87}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{6E89}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{6E8D}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{6E8F}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{6E90}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{6E98}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{6E9A}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{6E9C}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{6E9E}',
        readings: &["sao"],
    },
    CharPinyinEntry {
        ch: '\u{6E9F}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{6EA0}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{6EA2}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6EA5}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{6EA6}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{6EA7}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{6EAA}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6EAF}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{6EB1}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{6EB2}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{6EB4}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{6EB5}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{6EB6}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{6EB7}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{6EB9}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{6EBA}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{6EBB}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{6EBD}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{6EC1}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{6EC2}',
        readings: &["pang"],
    },
    CharPinyinEntry {
        ch: '\u{6EC3}',
        readings: &["weng"],
    },
    CharPinyinEntry {
        ch: '\u{6EC6}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{6EC7}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{6EC9}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{6ECB}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{6ECD}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6ECF}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{6ED1}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{6ED3}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{6ED4}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{6ED5}',
        readings: &["teng"],
    },
    CharPinyinEntry {
        ch: '\u{6ED7}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{6ED8}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{6EDA}',
        readings: &["gun"],
    },
    CharPinyinEntry {
        ch: '\u{6EDE}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{6EDF}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{6EE0}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{6EE1}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{6EE2}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{6EE4}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{6EE5}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{6EE6}',
        readings: &["luan"],
    },
    CharPinyinEntry {
        ch: '\u{6EE7}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{6EE8}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{6EE9}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{6EEA}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6EEB}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{6EF4}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{6EF9}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{6F02}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{6F06}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{6F08}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6F09}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{6F0B}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{6F0F}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{6F13}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{6F14}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{6F15}',
        readings: &["cao"],
    },
    CharPinyinEntry {
        ch: '\u{6F16}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{6F20}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{6F24}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{6F26}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{6F29}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{6F2A}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6F2B}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{6F2D}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{6F2F}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{6F31}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6F33}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{6F34}',
        readings: &["chong", "shuang"],
    },
    CharPinyinEntry {
        ch: '\u{6F36}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{6F37}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{6F39}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{6F3B}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{6F3C}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{6F3E}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{6F46}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{6F47}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{6F4B}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{6F4D}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{6F4F}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{6F56}',
        readings: &["pa"],
    },
    CharPinyinEntry {
        ch: '\u{6F58}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{6F5C}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{6F5E}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{6F5F}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{6F62}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{6F66}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{6F69}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{6F6D}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{6F6E}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{6F72}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{6F74}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{6F75}',
        readings: &["sa"],
    },
    CharPinyinEntry {
        ch: '\u{6F78}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{6F7A}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{6F7C}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{6F7D}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{6F7E}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{6F82}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{6F84}',
        readings: &["cheng", "deng"],
    },
    CharPinyinEntry {
        ch: '\u{6F88}',
        readings: &["che"],
    },
    CharPinyinEntry {
        ch: '\u{6F89}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{6F8C}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{6F8D}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{6F8E}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{6F9B}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{6F9C}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{6FA1}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{6FA5}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{6FA7}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{6FAA}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{6FAD}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{6FB3}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{6FB4}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{6FB6}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{6FB9}',
        readings: &["dan", "tan"],
    },
    CharPinyinEntry {
        ch: '\u{6FBC}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{6FBD}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{6FC0}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{6FC2}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{6FC9}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{6FCB}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{6FD1}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{6FD2}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{6FDE}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{6FE0}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{6FE1}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{6FE9}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{6FEE}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{6FEF}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{700C}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{700D}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{7011}',
        readings: &["bao", "pu"],
    },
    CharPinyinEntry {
        ch: '\u{7014}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{701A}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{701B}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{7023}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{7031}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7035}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{7039}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{703C}',
        readings: &["rang"],
    },
    CharPinyinEntry {
        ch: '\u{7048}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{704C}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{704F}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{705E}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{706B}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{706D}',
        readings: &["mie"],
    },
    CharPinyinEntry {
        ch: '\u{706F}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{7070}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{7075}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{7076}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{7078}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{707C}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{707E}',
        readings: &["zai"],
    },
    CharPinyinEntry {
        ch: '\u{707F}',
        readings: &["can"],
    },
    CharPinyinEntry {
        ch: '\u{7080}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{7085}',
        readings: &["gui", "jiong"],
    },
    CharPinyinEntry {
        ch: '\u{7086}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{7089}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{708A}',
        readings: &["chui"],
    },
    CharPinyinEntry {
        ch: '\u{708C}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{708E}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{7092}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{7094}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{7095}',
        readings: &["kang"],
    },
    CharPinyinEntry {
        ch: '\u{7096}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{7098}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{7099}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{709C}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{709D}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{709F}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{70A3}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{70AB}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{70AC}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{70AD}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{70AE}',
        readings: &["bao", "pao"],
    },
    CharPinyinEntry {
        ch: '\u{70AF}',
        readings: &["jiong"],
    },
    CharPinyinEntry {
        ch: '\u{70B1}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{70B3}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{70B7}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{70B8}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{70B9}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{70BB}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{70BC}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{70BD}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{70C0}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{70C1}',
        readings: &["shuo"],
    },
    CharPinyinEntry {
        ch: '\u{70C2}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{70C3}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{70C8}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{70CA}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{70D4}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{70D8}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{70D9}',
        readings: &["lao", "luo"],
    },
    CharPinyinEntry {
        ch: '\u{70DB}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{70DC}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{70DD}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{70DF}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{70E0}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{70E4}',
        readings: &["kao"],
    },
    CharPinyinEntry {
        ch: '\u{70E6}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{70E7}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{70E8}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{70E9}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{70EB}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{70EC}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{70ED}',
        readings: &["re"],
    },
    CharPinyinEntry {
        ch: '\u{70EF}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{70F6}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{70F7}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{70F9}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{70FA}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{70FB}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{70FD}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{7106}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{7109}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{710A}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{710C}',
        readings: &["jun", "qu"],
    },
    CharPinyinEntry {
        ch: '\u{7110}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{7113}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{7115}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{7116}',
        readings: &["men"],
    },
    CharPinyinEntry {
        ch: '\u{7117}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{7118}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{7119}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{711A}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{711C}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{711E}',
        readings: &["tun"],
    },
    CharPinyinEntry {
        ch: '\u{7126}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{712F}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{7130}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{7131}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{7136}',
        readings: &["ran"],
    },
    CharPinyinEntry {
        ch: '\u{7141}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{7143}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{7145}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{714A}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{714B}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{714C}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{714E}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{7153}',
        readings: &["tuan"],
    },
    CharPinyinEntry {
        ch: '\u{715C}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{715E}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{715F}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{7164}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{7166}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{7167}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{7168}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{716E}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{7172}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{7173}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{7174}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{7178}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{717A}',
        readings: &["tui"],
    },
    CharPinyinEntry {
        ch: '\u{717D}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{7184}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7187}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{718A}',
        readings: &["xiong"],
    },
    CharPinyinEntry {
        ch: '\u{718F}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{7194}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{7198}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{7199}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{719B}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{719C}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{719F}',
        readings: &["shou", "shu"],
    },
    CharPinyinEntry {
        ch: '\u{71A0}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{71A5}',
        readings: &["teng"],
    },
    CharPinyinEntry {
        ch: '\u{71A8}',
        readings: &["yu", "yun"],
    },
    CharPinyinEntry {
        ch: '\u{71AC}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{71B5}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{71B9}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{71BB}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{71C3}',
        readings: &["ran"],
    },
    CharPinyinEntry {
        ch: '\u{71CA}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{71CB}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{71CE}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{71CF}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{71D4}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{71D5}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{71DA}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{71E0}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{71E5}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{71E7}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{71EE}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{71F9}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{7206}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{7207}',
        readings: &["ruo"],
    },
    CharPinyinEntry {
        ch: '\u{7214}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{721A}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{721D}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{721F}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{7228}',
        readings: &["cuan"],
    },
    CharPinyinEntry {
        ch: '\u{722A}',
        readings: &["zhao", "zhua"],
    },
    CharPinyinEntry {
        ch: '\u{722C}',
        readings: &["pa"],
    },
    CharPinyinEntry {
        ch: '\u{7230}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{7231}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{7235}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{7236}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7237}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{7238}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{7239}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{723B}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{723D}',
        readings: &["shuang"],
    },
    CharPinyinEntry {
        ch: '\u{723F}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{7241}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{7242}',
        readings: &["zang"],
    },
    CharPinyinEntry {
        ch: '\u{7247}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{7248}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{724C}',
        readings: &["pai"],
    },
    CharPinyinEntry {
        ch: '\u{724D}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{7252}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{7256}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{7259}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{725A}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{725B}',
        readings: &["niu"],
    },
    CharPinyinEntry {
        ch: '\u{725D}',
        readings: &["pin"],
    },
    CharPinyinEntry {
        ch: '\u{725F}',
        readings: &["mou", "mu"],
    },
    CharPinyinEntry {
        ch: '\u{7261}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{7262}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{7264}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{7265}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{7266}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{7267}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{7269}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{726E}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{726F}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{7272}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{7275}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{7279}',
        readings: &["te"],
    },
    CharPinyinEntry {
        ch: '\u{727A}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{727B}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{727E}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{727F}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{7280}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7281}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7284}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7287}',
        readings: &["ben"],
    },
    CharPinyinEntry {
        ch: '\u{728A}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{728B}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{728D}',
        readings: &["jian", "qian"],
    },
    CharPinyinEntry {
        ch: '\u{728F}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{7292}',
        readings: &["kao"],
    },
    CharPinyinEntry {
        ch: '\u{729F}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{72A8}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{72AC}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{72AF}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{72B0}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{72B4}',
        readings: &["an", "han"],
    },
    CharPinyinEntry {
        ch: '\u{72B6}',
        readings: &["zhuang"],
    },
    CharPinyinEntry {
        ch: '\u{72B7}',
        readings: &["guang"],
    },
    CharPinyinEntry {
        ch: '\u{72B8}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{72B9}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{72C1}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{72C2}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{72C3}',
        readings: &["niu"],
    },
    CharPinyinEntry {
        ch: '\u{72C4}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{72C8}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{72C9}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{72CD}',
        readings: &["pao"],
    },
    CharPinyinEntry {
        ch: '\u{72CE}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{72D0}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{72D2}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{72D7}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{72D9}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{72DD}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{72DE}',
        readings: &["ning"],
    },
    CharPinyinEntry {
        ch: '\u{72E0}',
        readings: &["hen"],
    },
    CharPinyinEntry {
        ch: '\u{72E1}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{72E8}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{72E9}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{72EC}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{72ED}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{72EE}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{72EF}',
        readings: &["kuai"],
    },
    CharPinyinEntry {
        ch: '\u{72F0}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{72F1}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{72F2}',
        readings: &["sun"],
    },
    CharPinyinEntry {
        ch: '\u{72F3}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{72F4}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{72F7}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{72F8}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{72FA}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{72FB}',
        readings: &["suan"],
    },
    CharPinyinEntry {
        ch: '\u{72FC}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{7301}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7303}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{7304}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{7307}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{730A}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{730E}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{7315}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{7316}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{7317}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{731B}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{731C}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{731D}',
        readings: &["cu"],
    },
    CharPinyinEntry {
        ch: '\u{731E}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{7321}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{7322}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{7325}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{7329}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{732A}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{732B}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{732C}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{732E}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{732F}',
        readings: &["tuan"],
    },
    CharPinyinEntry {
        ch: '\u{7330}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{7331}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{7334}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{7337}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{7339}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{733A}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{733E}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{733F}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{734D}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{7350}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{7352}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{7357}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{7360}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{736C}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{736D}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{736F}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{7374}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{737E}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{7383}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{7384}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{7387}',
        readings: &["lv", "shuai"],
    },
    CharPinyinEntry {
        ch: '\u{7389}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{738B}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{738E}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{7391}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7392}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{7393}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{7395}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{7396}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{7398}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{7399}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{739A}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{739B}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{739E}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{739F}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{73A0}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{73A1}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{73A2}',
        readings: &["bin", "fen"],
    },
    CharPinyinEntry {
        ch: '\u{73A4}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{73A5}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{73A6}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{73A9}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{73AB}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{73AD}',
        readings: &["pin"],
    },
    CharPinyinEntry {
        ch: '\u{73AE}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{73AF}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{73B0}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{73B1}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{73B2}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{73B3}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{73B6}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{73B7}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{73B9}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{73BA}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{73BB}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{73BC}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{73BF}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{73C0}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{73C2}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{73C5}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{73C7}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{73C8}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{73C9}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{73CA}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{73CB}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{73CC}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{73CD}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{73CF}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{73D0}',
        readings: &["fa"],
    },
    CharPinyinEntry {
        ch: '\u{73D1}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{73D2}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{73D5}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{73D6}',
        readings: &["guang"],
    },
    CharPinyinEntry {
        ch: '\u{73D9}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{73DB}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{73DD}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{73DE}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{73E0}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{73E2}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{73E3}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{73E5}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{73E6}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{73E7}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{73E9}',
        readings: &["heng"],
    },
    CharPinyinEntry {
        ch: '\u{73EA}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{73EB}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{73ED}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{73F0}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{73F2}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{73F5}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{73F7}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{73F8}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{73F9}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{73FA}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{73FD}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{7400}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{7403}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{7404}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{7405}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{7406}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7407}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{7408}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7409}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{740A}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{740E}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{740F}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{7410}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{7414}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{741A}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{741B}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{741F}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{7421}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{7422}',
        readings: &["zhuo", "zuo"],
    },
    CharPinyinEntry {
        ch: '\u{7424}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{7425}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{7426}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{7428}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{742A}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{742B}',
        readings: &["beng"],
    },
    CharPinyinEntry {
        ch: '\u{742C}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{742D}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{742E}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{742F}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{7430}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{7432}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{7433}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{7434}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{7435}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{7436}',
        readings: &["pa"],
    },
    CharPinyinEntry {
        ch: '\u{743C}',
        readings: &["qiong"],
    },
    CharPinyinEntry {
        ch: '\u{7440}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{7441}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{7442}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{7443}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{7444}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{7445}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{7446}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{7451}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{7453}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{7454}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{7455}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{7456}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{7457}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{7459}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{745A}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{745B}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{745C}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{745D}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{745E}',
        readings: &["rui"],
    },
    CharPinyinEntry {
        ch: '\u{745F}',
        readings: &["se"],
    },
    CharPinyinEntry {
        ch: '\u{7462}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{7467}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{7468}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{746C}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{746D}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{7470}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{7471}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{7473}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{7476}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{7477}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{747E}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{7480}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{7481}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{7483}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7486}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{7487}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{7488}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{748B}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{748E}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{7490}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{7492}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{7498}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{749C}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{749E}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{749F}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{74A0}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{74A5}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{74A7}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{74A8}',
        readings: &["can"],
    },
    CharPinyinEntry {
        ch: '\u{74A9}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{74AA}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{74AC}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{74AE}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{74B1}',
        readings: &["se"],
    },
    CharPinyinEntry {
        ch: '\u{74B2}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{74BA}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{74C0}',
        readings: &["ruan"],
    },
    CharPinyinEntry {
        ch: '\u{74D2}',
        readings: &["zan"],
    },
    CharPinyinEntry {
        ch: '\u{74D6}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{74D8}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{74DC}',
        readings: &["gua"],
    },
    CharPinyinEntry {
        ch: '\u{74DE}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{74E0}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{74E2}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{74E3}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{74E4}',
        readings: &["rang"],
    },
    CharPinyinEntry {
        ch: '\u{74E6}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{74EE}',
        readings: &["weng"],
    },
    CharPinyinEntry {
        ch: '\u{74EF}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{74F4}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{74F6}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{74F7}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{74FB}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{74FF}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{7504}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{750D}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{750F}',
        readings: &["beng"],
    },
    CharPinyinEntry {
        ch: '\u{7511}',
        readings: &["zeng"],
    },
    CharPinyinEntry {
        ch: '\u{7513}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{7517}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{7518}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{751A}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{751C}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{751F}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{7521}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{7525}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{7526}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{7528}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{7529}',
        readings: &["shuai"],
    },
    CharPinyinEntry {
        ch: '\u{752A}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{752B}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{752C}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{752D}',
        readings: &["beng"],
    },
    CharPinyinEntry {
        ch: '\u{752F}',
        readings: &["ning"],
    },
    CharPinyinEntry {
        ch: '\u{7530}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{7531}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{7532}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{7533}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{7535}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{7537}',
        readings: &["nan"],
    },
    CharPinyinEntry {
        ch: '\u{7538}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{753A}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{753B}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{753E}',
        readings: &["zai"],
    },
    CharPinyinEntry {
        ch: '\u{7540}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{7545}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{7548}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{754B}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{754C}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{754E}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{754F}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{7554}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{7556}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{7559}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{755A}',
        readings: &["ben"],
    },
    CharPinyinEntry {
        ch: '\u{755B}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{755C}',
        readings: &["chu", "xu"],
    },
    CharPinyinEntry {
        ch: '\u{7564}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{7565}',
        readings: &["lve"],
    },
    CharPinyinEntry {
        ch: '\u{7566}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{756A}',
        readings: &["fan", "pan"],
    },
    CharPinyinEntry {
        ch: '\u{756C}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{756F}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{7572}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{7574}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{7578}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7579}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{757F}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7581}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{7583}',
        readings: &["tuan"],
    },
    CharPinyinEntry {
        ch: '\u{7586}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{758D}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{758F}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{7590}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{7591}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7594}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{7596}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{7597}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{7599}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{759A}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{759D}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{759F}',
        readings: &["nve", "yao"],
    },
    CharPinyinEntry {
        ch: '\u{75A0}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{75A1}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{75A2}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{75A3}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{75A4}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{75A5}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{75AB}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{75AC}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{75AD}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{75AE}',
        readings: &["chuang"],
    },
    CharPinyinEntry {
        ch: '\u{75AF}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{75B0}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{75B1}',
        readings: &["pao"],
    },
    CharPinyinEntry {
        ch: '\u{75B2}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{75B3}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{75B4}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{75B5}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{75B8}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{75B9}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{75BC}',
        readings: &["teng"],
    },
    CharPinyinEntry {
        ch: '\u{75BD}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{75BE}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{75C2}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{75C3}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{75C4}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{75C5}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{75C7}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{75C8}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{75C9}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{75CA}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{75CD}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{75D2}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{75D3}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{75D4}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{75D5}',
        readings: &["hen"],
    },
    CharPinyinEntry {
        ch: '\u{75D8}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{75DB}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{75DE}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{75E2}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{75E3}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{75E4}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{75E6}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{75E7}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{75E8}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{75EA}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{75EB}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{75F0}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{75F1}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{75F4}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{75F9}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{75FC}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{75FF}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{7600}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{7601}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{7603}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{7605}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{7606}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{760A}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{760C}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{7610}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{7615}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{7617}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7618}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{7619}',
        readings: &["sao"],
    },
    CharPinyinEntry {
        ch: '\u{761B}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{761F}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{7620}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7622}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{7624}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{7625}',
        readings: &["chai"],
    },
    CharPinyinEntry {
        ch: '\u{7626}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{7629}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{762A}',
        readings: &["bie"],
    },
    CharPinyinEntry {
        ch: '\u{762B}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{762D}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{7630}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{7633}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{7634}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{7635}',
        readings: &["zhai"],
    },
    CharPinyinEntry {
        ch: '\u{7638}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{763C}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{763E}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{763F}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{7640}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{7643}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{764C}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{764D}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{7654}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7656}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{7657}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{765C}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{765E}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{7663}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{766B}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{766F}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{7678}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{767B}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{767D}',
        readings: &["bai"],
    },
    CharPinyinEntry {
        ch: '\u{767E}',
        readings: &["bai"],
    },
    CharPinyinEntry {
        ch: '\u{767F}',
        readings: &["bie"],
    },
    CharPinyinEntry {
        ch: '\u{7682}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{7684}',
        readings: &["de", "di"],
    },
    CharPinyinEntry {
        ch: '\u{7686}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{7687}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{7688}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{768B}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{768E}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{7691}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{7693}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{7695}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{7696}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{7699}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{769B}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{769E}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{76A4}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{76A6}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{76AD}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{76AE}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{76B1}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{76B2}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{76B4}',
        readings: &["cun"],
    },
    CharPinyinEntry {
        ch: '\u{76BF}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{76C2}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{76C5}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{76C6}',
        readings: &["pen"],
    },
    CharPinyinEntry {
        ch: '\u{76C8}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{76C9}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{76CA}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{76CD}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{76CE}',
        readings: &["ang"],
    },
    CharPinyinEntry {
        ch: '\u{76CF}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{76D0}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{76D1}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{76D2}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{76D4}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{76D6}',
        readings: &["gai", "ge"],
    },
    CharPinyinEntry {
        ch: '\u{76D7}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{76D8}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{76DB}',
        readings: &["cheng", "sheng"],
    },
    CharPinyinEntry {
        ch: '\u{76DF}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{76E5}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{76E6}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{76EE}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{76EF}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{76F1}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{76F2}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{76F4}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{76F7}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{76F8}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{76F9}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{76FC}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{76FE}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{7701}',
        readings: &["sheng", "xing"],
    },
    CharPinyinEntry {
        ch: '\u{7704}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{7707}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{7708}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{7709}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{770A}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{770B}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{770D}',
        readings: &["kou"],
    },
    CharPinyinEntry {
        ch: '\u{7719}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{771A}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{771F}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{7720}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{7722}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{7726}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{7728}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{7729}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{772C}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{772D}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{772F}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{7735}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{7736}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{7737}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{7738}',
        readings: &["mou"],
    },
    CharPinyinEntry {
        ch: '\u{773A}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{773C}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{7740}',
        readings: &["zhao", "zhe", "zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{7741}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{7743}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{7744}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{7747}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{774E}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7750}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{7751}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{775A}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{775B}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{7761}',
        readings: &["shui"],
    },
    CharPinyinEntry {
        ch: '\u{7762}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{7763}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{7765}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{7766}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{7768}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{776B}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{776C}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{7779}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{777D}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{777E}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{777F}',
        readings: &["rui"],
    },
    CharPinyinEntry {
        ch: '\u{7780}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{7784}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{7785}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{778B}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{778C}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{778D}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{778E}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{7791}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{7792}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{779F}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{77A0}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{77A2}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{77A5}',
        readings: &["pie"],
    },
    CharPinyinEntry {
        ch: '\u{77A7}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{77A9}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{77AA}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{77AB}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{77AC}',
        readings: &["shun"],
    },
    CharPinyinEntry {
        ch: '\u{77AD}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{77B0}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{77B3}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{77B5}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{77BB}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{77BD}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{77BF}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{77CD}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{77D7}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{77DB}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{77DC}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{77DE}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{77E2}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{77E3}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{77E5}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{77E7}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{77E9}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{77EB}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{77EC}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{77ED}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{77EE}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{77F0}',
        readings: &["zeng"],
    },
    CharPinyinEntry {
        ch: '\u{77F3}',
        readings: &["dan", "shi"],
    },
    CharPinyinEntry {
        ch: '\u{77F6}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{77F8}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{77FB}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{77FC}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{77FE}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{77FF}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{7800}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{7801}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{7802}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{7804}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{7806}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7809}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{780C}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{780D}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{7811}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{7812}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{7814}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{7816}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{7817}',
        readings: &["che"],
    },
    CharPinyinEntry {
        ch: '\u{7818}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{781A}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{781C}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{781D}',
        readings: &["fa"],
    },
    CharPinyinEntry {
        ch: '\u{781F}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{7820}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{7823}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{7825}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{7827}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{782B}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{782C}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{782D}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{782E}',
        readings: &["nu"],
    },
    CharPinyinEntry {
        ch: '\u{7830}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{7834}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{7835}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{7837}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{7838}',
        readings: &["za"],
    },
    CharPinyinEntry {
        ch: '\u{7839}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{783A}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{783B}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{783C}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{783E}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7840}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{7841}',
        readings: &["keng"],
    },
    CharPinyinEntry {
        ch: '\u{7845}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{7847}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{784A}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{784C}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{784D}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{784E}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{7850}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{7852}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7854}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{7855}',
        readings: &["shuo"],
    },
    CharPinyinEntry {
        ch: '\u{7856}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{7857}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{7859}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{785A}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{785D}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{786A}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{786B}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{786C}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{786D}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{786E}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{787C}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{787F}',
        readings: &["kong"],
    },
    CharPinyinEntry {
        ch: '\u{7883}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{7887}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{7888}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{7889}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{788C}',
        readings: &["liu", "lu"],
    },
    CharPinyinEntry {
        ch: '\u{788D}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{788E}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{788F}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{7891}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{7893}',
        readings: &["dui"],
    },
    CharPinyinEntry {
        ch: '\u{7897}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{7898}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{789A}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{789B}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{789C}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{789F}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{78A1}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{78A3}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{78A5}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{78A7}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{78A8}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{78B0}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{78B1}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{78B2}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{78B3}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{78B4}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{78B6}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{78B9}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{78BE}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{78C1}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{78C5}',
        readings: &["bang", "pang"],
    },
    CharPinyinEntry {
        ch: '\u{78C9}',
        readings: &["sang"],
    },
    CharPinyinEntry {
        ch: '\u{78CA}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{78CB}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{78CF}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{78D0}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{78D4}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{78D5}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{78D9}',
        readings: &["gun"],
    },
    CharPinyinEntry {
        ch: '\u{78DC}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{78E1}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{78E8}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{78EC}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{78F2}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{78F4}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{78F7}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{78F9}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{78FB}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{7901}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{7905}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{790C}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{7913}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{791E}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{7934}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{7935}',
        readings: &["shuang"],
    },
    CharPinyinEntry {
        ch: '\u{793A}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{793C}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{793E}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{7940}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{7941}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{7943}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{7946}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{7947}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{7948}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{7949}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{794A}',
        readings: &["beng"],
    },
    CharPinyinEntry {
        ch: '\u{794B}',
        readings: &["dui"],
    },
    CharPinyinEntry {
        ch: '\u{794E}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{794F}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{7950}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{7953}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7955}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{7956}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{7957}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{795A}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{795B}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{795C}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{795D}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{795E}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{795F}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{7960}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{7962}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{7965}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{7967}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{7968}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{796D}',
        readings: &["ji", "zhai"],
    },
    CharPinyinEntry {
        ch: '\u{796F}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{7972}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{7977}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{7978}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{797A}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{797C}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{797E}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{7980}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{7981}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{7984}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{7985}',
        readings: &["chan", "shan"],
    },
    CharPinyinEntry {
        ch: '\u{798A}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{798B}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{798F}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7992}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{7994}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{7998}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{799A}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{799B}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{79A4}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{79A7}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{79B3}',
        readings: &["rang"],
    },
    CharPinyinEntry {
        ch: '\u{79B9}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{79BA}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{79BB}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{79BD}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{79BE}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{79C0}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{79C1}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{79C3}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{79C6}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{79C9}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{79CB}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{79CD}',
        readings: &["chong", "zhong"],
    },
    CharPinyinEntry {
        ch: '\u{79D1}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{79D2}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{79D5}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{79D8}',
        readings: &["bi", "mi"],
    },
    CharPinyinEntry {
        ch: '\u{79DF}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{79E3}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{79E4}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{79E6}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{79E7}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{79E9}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{79EB}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{79EC}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{79ED}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{79EF}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{79F0}',
        readings: &["chen", "cheng"],
    },
    CharPinyinEntry {
        ch: '\u{79F8}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{79FB}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{79FD}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{79FE}',
        readings: &["nong"],
    },
    CharPinyinEntry {
        ch: '\u{7A00}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7A02}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{7A03}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7A06}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{7A0B}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{7A0C}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{7A0D}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{7A0E}',
        readings: &["shui"],
    },
    CharPinyinEntry {
        ch: '\u{7A11}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{7A14}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{7A17}',
        readings: &["bai"],
    },
    CharPinyinEntry {
        ch: '\u{7A19}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{7A1A}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{7A1E}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{7A20}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{7A23}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{7A33}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{7A37}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7A39}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{7A3B}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{7A3C}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{7A3D}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7A3F}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{7A44}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7A46}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{7A51}',
        readings: &["se"],
    },
    CharPinyinEntry {
        ch: '\u{7A57}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{7A59}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{7A5C}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{7A5F}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{7A70}',
        readings: &["rang"],
    },
    CharPinyinEntry {
        ch: '\u{7A74}',
        readings: &["xue"],
    },
    CharPinyinEntry {
        ch: '\u{7A76}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{7A77}',
        readings: &["qiong"],
    },
    CharPinyinEntry {
        ch: '\u{7A78}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7A79}',
        readings: &["qiong"],
    },
    CharPinyinEntry {
        ch: '\u{7A7A}',
        readings: &["kong"],
    },
    CharPinyinEntry {
        ch: '\u{7A7F}',
        readings: &["chuan"],
    },
    CharPinyinEntry {
        ch: '\u{7A80}',
        readings: &["zhun"],
    },
    CharPinyinEntry {
        ch: '\u{7A81}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{7A83}',
        readings: &["qie"],
    },
    CharPinyinEntry {
        ch: '\u{7A84}',
        readings: &["zhai"],
    },
    CharPinyinEntry {
        ch: '\u{7A85}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{7A88}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{7A8A}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{7A8D}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{7A8E}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{7A91}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{7A92}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{7A95}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{7A96}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{7A97}',
        readings: &["chuang"],
    },
    CharPinyinEntry {
        ch: '\u{7A98}',
        readings: &["jiong"],
    },
    CharPinyinEntry {
        ch: '\u{7A9C}',
        readings: &["cuan"],
    },
    CharPinyinEntry {
        ch: '\u{7A9D}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{7A9F}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{7AA0}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{7AA3}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{7AA5}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{7AA6}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{7AA8}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{7AAC}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{7AAD}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{7AB3}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{7AB8}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7ABF}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{7ACB}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7AD1}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{7AD6}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{7AD8}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{7AD9}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{7ADE}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{7ADF}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{7AE0}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{7AE3}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{7AE5}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{7AE6}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{7AEB}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{7AED}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{7AEF}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{7AF9}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{7AFA}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{7AFD}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{7AFF}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{7B03}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{7B04}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7B06}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{7B08}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7B0A}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{7B0B}',
        readings: &["sun"],
    },
    CharPinyinEntry {
        ch: '\u{7B0F}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{7B11}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{7B14}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{7B15}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{7B19}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{7B1B}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{7B1E}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{7B20}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7B24}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{7B25}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{7B26}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7B28}',
        readings: &["ben"],
    },
    CharPinyinEntry {
        ch: '\u{7B2A}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{7B2B}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{7B2C}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{7B2E}',
        readings: &["ze", "zuo"],
    },
    CharPinyinEntry {
        ch: '\u{7B2F}',
        readings: &["nu"],
    },
    CharPinyinEntry {
        ch: '\u{7B31}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{7B33}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{7B38}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{7B3A}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{7B3C}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{7B3E}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{7B40}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{7B45}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{7B47}',
        readings: &["qiong"],
    },
    CharPinyinEntry {
        ch: '\u{7B49}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{7B4B}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{7B4C}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{7B4F}',
        readings: &["fa"],
    },
    CharPinyinEntry {
        ch: '\u{7B50}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{7B51}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{7B52}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{7B54}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{7B56}',
        readings: &["ce"],
    },
    CharPinyinEntry {
        ch: '\u{7B58}',
        readings: &["kou"],
    },
    CharPinyinEntry {
        ch: '\u{7B5A}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{7B5B}',
        readings: &["shai"],
    },
    CharPinyinEntry {
        ch: '\u{7B5C}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{7B5D}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{7B60}',
        readings: &["jun", "yun"],
    },
    CharPinyinEntry {
        ch: '\u{7B62}',
        readings: &["pa"],
    },
    CharPinyinEntry {
        ch: '\u{7B64}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{7B65}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{7B66}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{7B6E}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{7B71}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{7B72}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{7B75}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{7B76}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{7B77}',
        readings: &["kuai"],
    },
    CharPinyinEntry {
        ch: '\u{7B79}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{7B7B}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{7B7C}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{7B7E}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{7B80}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{7B85}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{7B8D}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{7B90}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{7B93}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{7B94}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{7B95}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7B96}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{7B97}',
        readings: &["suan"],
    },
    CharPinyinEntry {
        ch: '\u{7B9C}',
        readings: &["kong"],
    },
    CharPinyinEntry {
        ch: '\u{7BA1}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{7BA2}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{7BA6}',
        readings: &["ze"],
    },
    CharPinyinEntry {
        ch: '\u{7BA7}',
        readings: &["qie"],
    },
    CharPinyinEntry {
        ch: '\u{7BA8}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{7BA9}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{7BAA}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{7BAB}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{7BAC}',
        readings: &["ruo"],
    },
    CharPinyinEntry {
        ch: '\u{7BAD}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{7BB1}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{7BB4}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{7BB8}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{7BC1}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{7BC6}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{7BC7}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{7BCC}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{7BD1}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{7BD3}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{7BD9}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{7BDA}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{7BDD}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{7BE1}',
        readings: &["cuan"],
    },
    CharPinyinEntry {
        ch: '\u{7BE5}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7BE6}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{7BEA}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{7BEE}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{7BEF}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{7BF1}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7BF7}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{7BFC}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{7BFE}',
        readings: &["mie"],
    },
    CharPinyinEntry {
        ch: '\u{7C03}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7C07}',
        readings: &["cu"],
    },
    CharPinyinEntry {
        ch: '\u{7C09}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{7C0B}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{7C0C}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{7C0F}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{7C15}',
        readings: &["le"],
    },
    CharPinyinEntry {
        ch: '\u{7C16}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{7C1D}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{7C1F}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{7C20}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7C27}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{7C2A}',
        readings: &["zan"],
    },
    CharPinyinEntry {
        ch: '\u{7C30}',
        readings: &["pai"],
    },
    CharPinyinEntry {
        ch: '\u{7C38}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{7C3F}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{7C40}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{7C41}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{7C4D}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7C65}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{7C73}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{7C74}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{7C7B}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{7C7C}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{7C7D}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{7C89}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{7C91}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{7C92}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7C95}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{7C97}',
        readings: &["cu"],
    },
    CharPinyinEntry {
        ch: '\u{7C98}',
        readings: &["nian", "zhan"],
    },
    CharPinyinEntry {
        ch: '\u{7C9C}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{7C9D}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7C9E}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7C9F}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{7CA2}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{7CA4}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{7CA5}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{7CAA}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{7CAE}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{7CB1}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{7CB2}',
        readings: &["can"],
    },
    CharPinyinEntry {
        ch: '\u{7CB3}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{7CB9}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{7CBC}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{7CBD}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{7CBE}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{7CBF}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{7CC1}',
        readings: &["san", "shen"],
    },
    CharPinyinEntry {
        ch: '\u{7CC5}',
        readings: &["rou"],
    },
    CharPinyinEntry {
        ch: '\u{7CC7}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{7CC8}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{7CCA}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{7CCC}',
        readings: &["zan"],
    },
    CharPinyinEntry {
        ch: '\u{7CCD}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{7CD2}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{7CD5}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{7CD6}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{7CD7}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{7CD9}',
        readings: &["cao"],
    },
    CharPinyinEntry {
        ch: '\u{7CDC}',
        readings: &["mei", "mi"],
    },
    CharPinyinEntry {
        ch: '\u{7CDF}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{7CE0}',
        readings: &["kang"],
    },
    CharPinyinEntry {
        ch: '\u{7CE8}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{7CEF}',
        readings: &["nuo"],
    },
    CharPinyinEntry {
        ch: '\u{7CF5}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{7CFB}',
        readings: &["ji", "xi"],
    },
    CharPinyinEntry {
        ch: '\u{7D0A}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{7D20}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{7D22}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{7D27}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{7D2B}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{7D2F}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{7D5C}',
        readings: &["jie", "xie"],
    },
    CharPinyinEntry {
        ch: '\u{7D6E}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{7D77}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{7DA6}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{7DAE}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{7E20}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{7E22}',
        readings: &["teng"],
    },
    CharPinyinEntry {
        ch: '\u{7E3B}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{7E41}',
        readings: &["fan", "po"],
    },
    CharPinyinEntry {
        ch: '\u{7E44}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7E47}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{7E82}',
        readings: &["zuan"],
    },
    CharPinyinEntry {
        ch: '\u{7E9B}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{7EA0}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{7EA1}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{7EA2}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{7EA3}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{7EA4}',
        readings: &["qian", "xian"],
    },
    CharPinyinEntry {
        ch: '\u{7EA5}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{7EA6}',
        readings: &["yao", "yue"],
    },
    CharPinyinEntry {
        ch: '\u{7EA7}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7EA8}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{7EA9}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{7EAA}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7EAB}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{7EAC}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{7EAD}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{7EAE}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{7EAF}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{7EB0}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{7EB1}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{7EB2}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{7EB3}',
        readings: &["na"],
    },
    CharPinyinEntry {
        ch: '\u{7EB4}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{7EB5}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{7EB6}',
        readings: &["guan", "lun"],
    },
    CharPinyinEntry {
        ch: '\u{7EB7}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{7EB8}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{7EB9}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{7EBA}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{7EBB}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{7EBC}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{7EBD}',
        readings: &["niu"],
    },
    CharPinyinEntry {
        ch: '\u{7EBE}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{7EBF}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{7EC0}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{7EC1}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{7EC2}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7EC3}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{7EC4}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{7EC5}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{7EC6}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7EC7}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{7EC8}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{7EC9}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{7ECA}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{7ECB}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7ECC}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{7ECD}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{7ECE}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7ECF}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{7ED0}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{7ED1}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{7ED2}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{7ED3}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{7ED4}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{7ED5}',
        readings: &["rao"],
    },
    CharPinyinEntry {
        ch: '\u{7ED6}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{7ED7}',
        readings: &["hang"],
    },
    CharPinyinEntry {
        ch: '\u{7ED8}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{7ED9}',
        readings: &["gei", "ji"],
    },
    CharPinyinEntry {
        ch: '\u{7EDA}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{7EDB}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{7EDC}',
        readings: &["lao", "luo"],
    },
    CharPinyinEntry {
        ch: '\u{7EDD}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{7EDE}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{7EDF}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{7EE0}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{7EE1}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{7EE2}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{7EE3}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{7EE4}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7EE5}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{7EE6}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{7EE7}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7EE8}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{7EE9}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7EEA}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{7EEB}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{7EED}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{7EEE}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{7EEF}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{7EF0}',
        readings: &["chao", "chuo"],
    },
    CharPinyinEntry {
        ch: '\u{7EF1}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{7EF2}',
        readings: &["gun"],
    },
    CharPinyinEntry {
        ch: '\u{7EF3}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{7EF4}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{7EF5}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{7EF6}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{7EF7}',
        readings: &["beng"],
    },
    CharPinyinEntry {
        ch: '\u{7EF8}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{7EF9}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{7EFA}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{7EFB}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{7EFC}',
        readings: &["zeng", "zong"],
    },
    CharPinyinEntry {
        ch: '\u{7EFD}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{7EFE}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{7EFF}',
        readings: &["lu", "lv"],
    },
    CharPinyinEntry {
        ch: '\u{7F00}',
        readings: &["zhui"],
    },
    CharPinyinEntry {
        ch: '\u{7F01}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{7F02}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{7F03}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{7F04}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{7F05}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{7F06}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{7F07}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{7F08}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{7F09}',
        readings: &["ji", "qi"],
    },
    CharPinyinEntry {
        ch: '\u{7F0A}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{7F0C}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{7F0E}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{7F10}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{7F11}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{7F12}',
        readings: &["zhui"],
    },
    CharPinyinEntry {
        ch: '\u{7F13}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{7F14}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{7F15}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{7F16}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{7F17}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{7F18}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{7F19}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{7F1A}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7F1B}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{7F1C}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{7F1D}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{7F1E}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{7F1F}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{7F20}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{7F21}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7F22}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7F23}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{7F24}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{7F25}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{7F26}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{7F27}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{7F28}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{7F29}',
        readings: &["su", "suo"],
    },
    CharPinyinEntry {
        ch: '\u{7F2A}',
        readings: &["miao", "miu", "mou"],
    },
    CharPinyinEntry {
        ch: '\u{7F2B}',
        readings: &["sao"],
    },
    CharPinyinEntry {
        ch: '\u{7F2C}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{7F2D}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{7F2E}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{7F2F}',
        readings: &["zeng"],
    },
    CharPinyinEntry {
        ch: '\u{7F30}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{7F31}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{7F32}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{7F33}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{7F34}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{7F35}',
        readings: &["zuan"],
    },
    CharPinyinEntry {
        ch: '\u{7F36}',
        readings: &["fou"],
    },
    CharPinyinEntry {
        ch: '\u{7F38}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{7F3A}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{7F42}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{7F44}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{7F45}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{7F4D}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{7F50}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{7F51}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{7F54}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{7F55}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{7F57}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{7F58}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{7F5A}',
        readings: &["fa"],
    },
    CharPinyinEntry {
        ch: '\u{7F5F}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{7F61}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{7F62}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{7F68}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{7F69}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{7F6A}',
        readings: &["zui"],
    },
    CharPinyinEntry {
        ch: '\u{7F6E}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{7F71}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{7F72}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{7F74}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{7F76}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{7F79}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{7F7D}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7F7E}',
        readings: &["zeng"],
    },
    CharPinyinEntry {
        ch: '\u{7F81}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{7F8A}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{7F8C}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{7F8E}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{7F91}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{7F93}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{7F94}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{7F95}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{7F96}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{7F9A}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{7F9D}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{7F9E}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{7F9F}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{7FA1}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{7FA4}',
        readings: &["qun"],
    },
    CharPinyinEntry {
        ch: '\u{7FA7}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{7FAF}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{7FB0}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{7FB1}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{7FB2}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7FB8}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{7FB9}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{7FBC}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{7FBD}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{7FBF}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7FC0}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{7FC1}',
        readings: &["weng"],
    },
    CharPinyinEntry {
        ch: '\u{7FC2}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{7FC3}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{7FC5}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{7FC8}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{7FCA}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7FCC}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7FCE}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{7FD4}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{7FD5}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{7FD8}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{7FD9}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{7FDA}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{7FDB}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{7FDF}',
        readings: &["di", "zhai"],
    },
    CharPinyinEntry {
        ch: '\u{7FE0}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{7FE1}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{7FE5}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{7FE6}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{7FE9}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{7FEE}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{7FEF}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{7FF0}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{7FF1}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{7FF3}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7FF7}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{7FFB}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{7FFC}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{7FFE}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{8000}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{8001}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{8003}',
        readings: &["kao"],
    },
    CharPinyinEntry {
        ch: '\u{8004}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{8005}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{8006}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{8007}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{800B}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{800C}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{800D}',
        readings: &["shua"],
    },
    CharPinyinEntry {
        ch: '\u{800F}',
        readings: &["er", "nai"],
    },
    CharPinyinEntry {
        ch: '\u{8010}',
        readings: &["nai"],
    },
    CharPinyinEntry {
        ch: '\u{8011}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{8012}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{8014}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{8015}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{8016}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{8017}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{8018}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{8019}',
        readings: &["ba", "pa"],
    },
    CharPinyinEntry {
        ch: '\u{801C}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{8020}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{8022}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{8024}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8025}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{8026}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{8027}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{8028}',
        readings: &["nou"],
    },
    CharPinyinEntry {
        ch: '\u{8029}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{802A}',
        readings: &["pang"],
    },
    CharPinyinEntry {
        ch: '\u{8030}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{8031}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{8033}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{8035}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{8036}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{8037}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{8038}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{803B}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{803D}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{803F}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{8042}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{8043}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{8046}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{804A}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{804B}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{804C}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{804D}',
        readings: &["ning"],
    },
    CharPinyinEntry {
        ch: '\u{8052}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{8054}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{8058}',
        readings: &["pin"],
    },
    CharPinyinEntry {
        ch: '\u{805A}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{8069}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{806A}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{8071}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{807F}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8083}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{8084}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8086}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{8087}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{8089}',
        readings: &["rou"],
    },
    CharPinyinEntry {
        ch: '\u{808B}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{808C}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8093}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{8096}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{8098}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{809A}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{809B}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{809D}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{809F}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{80A0}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{80A1}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{80A2}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{80A4}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{80A5}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{80A9}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{80AA}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{80AB}',
        readings: &["zhun"],
    },
    CharPinyinEntry {
        ch: '\u{80AD}',
        readings: &["na"],
    },
    CharPinyinEntry {
        ch: '\u{80AE}',
        readings: &["ang"],
    },
    CharPinyinEntry {
        ch: '\u{80AF}',
        readings: &["ken"],
    },
    CharPinyinEntry {
        ch: '\u{80B1}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{80B2}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{80B4}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{80B7}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{80B8}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{80BA}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{80BC}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{80BD}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{80BE}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{80BF}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{80C0}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{80C1}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{80C2}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{80C3}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{80C4}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{80C6}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{80C8}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{80CC}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{80CD}',
        readings: &["gua"],
    },
    CharPinyinEntry {
        ch: '\u{80CE}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{80D6}',
        readings: &["pan", "pang"],
    },
    CharPinyinEntry {
        ch: '\u{80D7}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{80D9}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{80DA}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{80DB}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{80DC}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{80DD}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{80DE}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{80E0}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{80E1}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{80E3}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{80E4}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{80E5}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{80E7}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{80E8}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{80E9}',
        readings: &["ka"],
    },
    CharPinyinEntry {
        ch: '\u{80EA}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{80EB}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{80EC}',
        readings: &["nu"],
    },
    CharPinyinEntry {
        ch: '\u{80ED}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{80EF}',
        readings: &["kua"],
    },
    CharPinyinEntry {
        ch: '\u{80F0}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{80F1}',
        readings: &["guang"],
    },
    CharPinyinEntry {
        ch: '\u{80F2}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{80F3}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{80F4}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{80F6}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{80F8}',
        readings: &["xiong"],
    },
    CharPinyinEntry {
        ch: '\u{80FA}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{80FC}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{80FD}',
        readings: &["neng"],
    },
    CharPinyinEntry {
        ch: '\u{8102}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8106}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{8109}',
        readings: &["mai", "mo"],
    },
    CharPinyinEntry {
        ch: '\u{810A}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{810D}',
        readings: &["kuai"],
    },
    CharPinyinEntry {
        ch: '\u{810E}',
        readings: &["sa"],
    },
    CharPinyinEntry {
        ch: '\u{810F}',
        readings: &["zang"],
    },
    CharPinyinEntry {
        ch: '\u{8110}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{8111}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{8112}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{8113}',
        readings: &["nong"],
    },
    CharPinyinEntry {
        ch: '\u{8114}',
        readings: &["luan"],
    },
    CharPinyinEntry {
        ch: '\u{8116}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{8118}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{811A}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{811E}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{811F}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{8129}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{812C}',
        readings: &["pao"],
    },
    CharPinyinEntry {
        ch: '\u{812F}',
        readings: &["fu", "pu"],
    },
    CharPinyinEntry {
        ch: '\u{8131}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{8132}',
        readings: &["niao"],
    },
    CharPinyinEntry {
        ch: '\u{8136}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{8138}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{813E}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{813F}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{8146}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{8148}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{814A}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{814B}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{814C}',
        readings: &["a", "yan"],
    },
    CharPinyinEntry {
        ch: '\u{8150}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8151}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8152}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{8153}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{8154}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{8155}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{8158}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{8159}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{815A}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{8160}',
        readings: &["cou"],
    },
    CharPinyinEntry {
        ch: '\u{8165}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{8167}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{8168}',
        readings: &["shuan"],
    },
    CharPinyinEntry {
        ch: '\u{8169}',
        readings: &["nan"],
    },
    CharPinyinEntry {
        ch: '\u{816D}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{816E}',
        readings: &["sai"],
    },
    CharPinyinEntry {
        ch: '\u{816F}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{8170}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{8171}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8174}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8179}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{817A}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{817B}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{817C}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{817D}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{817E}',
        readings: &["teng"],
    },
    CharPinyinEntry {
        ch: '\u{817F}',
        readings: &["tui"],
    },
    CharPinyinEntry {
        ch: '\u{8180}',
        readings: &["bang", "pang"],
    },
    CharPinyinEntry {
        ch: '\u{8182}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{8188}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{818A}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{818F}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{8191}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{8198}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{8199}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{819B}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{819C}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{819D}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{81A6}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{81A8}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{81B3}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{81BA}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{81BB}',
        readings: &["dan", "shan"],
    },
    CharPinyinEntry {
        ch: '\u{81C0}',
        readings: &["tun"],
    },
    CharPinyinEntry {
        ch: '\u{81C2}',
        readings: &["bei", "bi"],
    },
    CharPinyinEntry {
        ch: '\u{81C3}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{81C6}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{81CA}',
        readings: &["sao"],
    },
    CharPinyinEntry {
        ch: '\u{81CC}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{81D1}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{81DC}',
        readings: &["za"],
    },
    CharPinyinEntry {
        ch: '\u{81E3}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{81E7}',
        readings: &["zang"],
    },
    CharPinyinEntry {
        ch: '\u{81EA}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{81EC}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{81ED}',
        readings: &["chou", "xiu"],
    },
    CharPinyinEntry {
        ch: '\u{81F3}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{81F4}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{81FB}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{81FC}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{81FE}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8200}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{8201}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8202}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{8204}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{8205}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{8206}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{820C}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{820D}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{8210}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{8212}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{8214}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{821B}',
        readings: &["chuan"],
    },
    CharPinyinEntry {
        ch: '\u{821C}',
        readings: &["shun"],
    },
    CharPinyinEntry {
        ch: '\u{821E}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{821F}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{8220}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{8222}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{8223}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8225}',
        readings: &["pa"],
    },
    CharPinyinEntry {
        ch: '\u{822A}',
        readings: &["hang"],
    },
    CharPinyinEntry {
        ch: '\u{822B}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{822C}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{822D}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{822F}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{8230}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8231}',
        readings: &["cang"],
    },
    CharPinyinEntry {
        ch: '\u{8232}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{8233}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{8234}',
        readings: &["ze"],
    },
    CharPinyinEntry {
        ch: '\u{8235}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{8236}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{8237}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{8238}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{8239}',
        readings: &["chuan"],
    },
    CharPinyinEntry {
        ch: '\u{823B}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{823E}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{8244}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{8245}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8247}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{8249}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{824B}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{824E}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{824F}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{8258}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{825A}',
        readings: &["cao"],
    },
    CharPinyinEntry {
        ch: '\u{825F}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{8268}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{826E}',
        readings: &["gen"],
    },
    CharPinyinEntry {
        ch: '\u{826F}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{8270}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8272}',
        readings: &["se", "shai"],
    },
    CharPinyinEntry {
        ch: '\u{8273}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{8274}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{827A}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{827D}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{827E}',
        readings: &["ai", "yi"],
    },
    CharPinyinEntry {
        ch: '\u{827F}',
        readings: &["nai"],
    },
    CharPinyinEntry {
        ch: '\u{8282}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{8283}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{8284}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{8288}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{828A}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{828B}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{828D}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{828E}',
        readings: &["xiong"],
    },
    CharPinyinEntry {
        ch: '\u{828F}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{8291}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{8292}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{8297}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{8298}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{8299}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{829C}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{829D}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{829F}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{82A0}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{82A1}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{82A3}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{82A4}',
        readings: &["kou"],
    },
    CharPinyinEntry {
        ch: '\u{82A5}',
        readings: &["gai", "jie"],
    },
    CharPinyinEntry {
        ch: '\u{82A6}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{82A8}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{82A9}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{82AA}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{82AB}',
        readings: &["yan", "yuan"],
    },
    CharPinyinEntry {
        ch: '\u{82AC}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{82AD}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{82AE}',
        readings: &["rui"],
    },
    CharPinyinEntry {
        ch: '\u{82AF}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{82B0}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{82B1}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{82B3}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{82B4}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{82B7}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{82B8}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{82B9}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{82BC}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{82BD}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{82BE}',
        readings: &["fei", "fu"],
    },
    CharPinyinEntry {
        ch: '\u{82C1}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{82C4}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{82C7}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{82C8}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{82C9}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{82CA}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{82CB}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{82CC}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{82CD}',
        readings: &["cang"],
    },
    CharPinyinEntry {
        ch: '\u{82CE}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{82CF}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{82D1}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{82D2}',
        readings: &["ran"],
    },
    CharPinyinEntry {
        ch: '\u{82D3}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{82D4}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{82D5}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{82D7}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{82D8}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{82DB}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{82DC}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{82DE}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{82DF}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{82E0}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{82E1}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{82E3}',
        readings: &["ju", "qu"],
    },
    CharPinyinEntry {
        ch: '\u{82E4}',
        readings: &["pie"],
    },
    CharPinyinEntry {
        ch: '\u{82E5}',
        readings: &["ruo"],
    },
    CharPinyinEntry {
        ch: '\u{82E6}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{82E7}',
        readings: &["ning"],
    },
    CharPinyinEntry {
        ch: '\u{82EB}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{82EF}',
        readings: &["ben"],
    },
    CharPinyinEntry {
        ch: '\u{82F1}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{82F4}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{82F7}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{82F9}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{82FB}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{82FE}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{8300}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8301}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{8302}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{8303}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{8304}',
        readings: &["jia", "qie"],
    },
    CharPinyinEntry {
        ch: '\u{8305}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{8306}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{8308}',
        readings: &["ci", "zi"],
    },
    CharPinyinEntry {
        ch: '\u{8309}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{830B}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{830C}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{830E}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{830F}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{8311}',
        readings: &["niao"],
    },
    CharPinyinEntry {
        ch: '\u{8313}',
        readings: &["xue"],
    },
    CharPinyinEntry {
        ch: '\u{8314}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{8315}',
        readings: &["qiong"],
    },
    CharPinyinEntry {
        ch: '\u{8317}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{831A}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{831B}',
        readings: &["gen"],
    },
    CharPinyinEntry {
        ch: '\u{831C}',
        readings: &["qian", "xi"],
    },
    CharPinyinEntry {
        ch: '\u{831D}',
        readings: &["chai"],
    },
    CharPinyinEntry {
        ch: '\u{8327}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8328}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{832B}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{832C}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{832D}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{832F}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8331}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{8333}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{8334}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{8335}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{8336}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{8338}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{8339}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{833A}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{833C}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{833D}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{8340}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{8341}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{8343}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{8344}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{8346}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{8347}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{8349}',
        readings: &["cao"],
    },
    CharPinyinEntry {
        ch: '\u{834F}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{8350}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8351}',
        readings: &["ti", "yi"],
    },
    CharPinyinEntry {
        ch: '\u{8352}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{8353}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{8354}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{8356}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{8359}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{835A}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{835B}',
        readings: &["rao"],
    },
    CharPinyinEntry {
        ch: '\u{835C}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{835E}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{835F}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{8360}',
        readings: &["ji", "qi"],
    },
    CharPinyinEntry {
        ch: '\u{8361}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{8363}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{8364}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{8365}',
        readings: &["xing", "ying"],
    },
    CharPinyinEntry {
        ch: '\u{8366}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{8367}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{8368}',
        readings: &["qian", "xun"],
    },
    CharPinyinEntry {
        ch: '\u{8369}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{836A}',
        readings: &["sun"],
    },
    CharPinyinEntry {
        ch: '\u{836B}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{836C}',
        readings: &["mai"],
    },
    CharPinyinEntry {
        ch: '\u{836D}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{836E}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{836F}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{8377}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{8378}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{837B}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{837C}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{837D}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{8385}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{8386}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{8389}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{838E}',
        readings: &["sha", "suo"],
    },
    CharPinyinEntry {
        ch: '\u{8392}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{8393}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{8398}',
        readings: &["shen", "xin"],
    },
    CharPinyinEntry {
        ch: '\u{8399}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{839B}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{839C}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{839D}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{839E}',
        readings: &["guan", "wan"],
    },
    CharPinyinEntry {
        ch: '\u{83A0}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{83A8}',
        readings: &["lang", "liang"],
    },
    CharPinyinEntry {
        ch: '\u{83A9}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{83AA}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{83AB}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{83B0}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{83B1}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{83B2}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{83B3}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{83B4}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{83B6}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{83B7}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{83B8}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{83B9}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{83BA}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{83BC}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{83BD}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{83BF}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{83C0}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{83C1}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{83C2}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{83C5}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{83C7}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{83C9}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{83CA}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{83CC}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{83CD}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{83CF}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{83D4}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{83D6}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{83D8}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{83DC}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{83DD}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{83DF}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{83E0}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{83E1}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{83E5}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{83E9}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{83EA}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{83F0}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{83F1}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{83F2}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{83F9}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{83FC}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{83FD}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{8401}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{8403}',
        readings: &["cui"],
    },
    CharPinyinEntry {
        ch: '\u{8404}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{8406}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{840B}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{840C}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{840D}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{840E}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{840F}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{8411}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{8418}',
        readings: &["nai"],
    },
    CharPinyinEntry {
        ch: '\u{841A}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{841C}',
        readings: &["tie"],
    },
    CharPinyinEntry {
        ch: '\u{841D}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{8423}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{8424}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{8425}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{8426}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{8427}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{8428}',
        readings: &["sa"],
    },
    CharPinyinEntry {
        ch: '\u{8429}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{8431}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{8433}',
        readings: &["nan"],
    },
    CharPinyinEntry {
        ch: '\u{8438}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8439}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{843C}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{843D}',
        readings: &["la", "lao", "luo"],
    },
    CharPinyinEntry {
        ch: '\u{8446}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{844E}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{8451}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{8456}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{8457}',
        readings: &["zhu", "zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{8459}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{845A}',
        readings: &["ren", "shen"],
    },
    CharPinyinEntry {
        ch: '\u{845B}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{845C}',
        readings: &["qia"],
    },
    CharPinyinEntry {
        ch: '\u{8461}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{8463}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{8469}',
        readings: &["pa"],
    },
    CharPinyinEntry {
        ch: '\u{846B}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{846C}',
        readings: &["zang"],
    },
    CharPinyinEntry {
        ch: '\u{846D}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{8470}',
        readings: &["jun", "suo"],
    },
    CharPinyinEntry {
        ch: '\u{8471}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{8473}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{8474}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{8475}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{8476}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{8478}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{847A}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{8482}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{8484}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{8487}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{8488}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{8489}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{848B}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{848C}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{848E}',
        readings: &["pai"],
    },
    CharPinyinEntry {
        ch: '\u{8490}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{8497}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{8499}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{849C}',
        readings: &["suan"],
    },
    CharPinyinEntry {
        ch: '\u{849F}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{84A1}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{84A8}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{84AF}',
        readings: &["kuai"],
    },
    CharPinyinEntry {
        ch: '\u{84B1}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{84B2}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{84B4}',
        readings: &["shuo"],
    },
    CharPinyinEntry {
        ch: '\u{84B8}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{84B9}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{84BA}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{84BB}',
        readings: &["ruo"],
    },
    CharPinyinEntry {
        ch: '\u{84BD}',
        readings: &["en"],
    },
    CharPinyinEntry {
        ch: '\u{84BF}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{84C1}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{84C2}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{84C4}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{84C7}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{84C9}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{84CA}',
        readings: &["weng"],
    },
    CharPinyinEntry {
        ch: '\u{84CD}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{84CF}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{84D0}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{84D1}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{84D3}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{84D6}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{84DD}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{84DF}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{84E0}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{84E2}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{84E3}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{84E5}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{84E6}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{84EC}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{84F0}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{84FC}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{84FF}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{8500}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{8503}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{8508}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{850A}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{850C}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{8511}',
        readings: &["mie"],
    },
    CharPinyinEntry {
        ch: '\u{8513}',
        readings: &["man", "wan"],
    },
    CharPinyinEntry {
        ch: '\u{8517}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{851A}',
        readings: &["wei", "yu"],
    },
    CharPinyinEntry {
        ch: '\u{851F}',
        readings: &["cu"],
    },
    CharPinyinEntry {
        ch: '\u{8521}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{852B}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{852C}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{8537}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{8538}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{8539}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{853A}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{853B}',
        readings: &["kou"],
    },
    CharPinyinEntry {
        ch: '\u{853C}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{853D}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{8543}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{8548}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{8549}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{854A}',
        readings: &["rui"],
    },
    CharPinyinEntry {
        ch: '\u{8556}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{8557}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{8559}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{855E}',
        readings: &["zui"],
    },
    CharPinyinEntry {
        ch: '\u{8564}',
        readings: &["rui"],
    },
    CharPinyinEntry {
        ch: '\u{8568}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{8570}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{8572}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{8574}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{8579}',
        readings: &["weng"],
    },
    CharPinyinEntry {
        ch: '\u{857A}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{857B}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{857E}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{8581}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{8584}',
        readings: &["bao", "bo"],
    },
    CharPinyinEntry {
        ch: '\u{8585}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{8587}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{858F}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{859B}',
        readings: &["xue"],
    },
    CharPinyinEntry {
        ch: '\u{859C}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{85A2}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{85A4}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{85A8}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{85AA}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{85AE}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{85AF}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{85B0}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{85B3}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{85B7}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{85B8}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{85B9}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{85BF}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{85C1}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{85C9}',
        readings: &["ji", "jie"],
    },
    CharPinyinEntry {
        ch: '\u{85CF}',
        readings: &["cang", "zang"],
    },
    CharPinyinEntry {
        ch: '\u{85D0}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{85D3}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{85D5}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{85DC}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{85DF}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{85E0}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{85E4}',
        readings: &["teng"],
    },
    CharPinyinEntry {
        ch: '\u{85E6}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{85E8}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{85E9}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{85FB}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{85FF}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{8605}',
        readings: &["heng"],
    },
    CharPinyinEntry {
        ch: '\u{8611}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{8616}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{8618}',
        readings: &["rang"],
    },
    CharPinyinEntry {
        ch: '\u{8627}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{8629}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{8638}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{863C}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{864E}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{864F}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{8650}',
        readings: &["nve"],
    },
    CharPinyinEntry {
        ch: '\u{8651}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{8652}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{8653}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{8654}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{865A}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{865E}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8662}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{8664}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{866B}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{866C}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{866E}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8671}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{8677}',
        readings: &["gan", "han"],
    },
    CharPinyinEntry {
        ch: '\u{8678}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{8679}',
        readings: &["hong", "jiang"],
    },
    CharPinyinEntry {
        ch: '\u{867A}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{867B}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{867C}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{867D}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{867E}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{867F}',
        readings: &["chai"],
    },
    CharPinyinEntry {
        ch: '\u{8680}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{8681}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8682}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{8684}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{8686}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{868A}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{868B}',
        readings: &["rui"],
    },
    CharPinyinEntry {
        ch: '\u{868C}',
        readings: &["bang", "beng"],
    },
    CharPinyinEntry {
        ch: '\u{868D}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{8693}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{8695}',
        readings: &["can"],
    },
    CharPinyinEntry {
        ch: '\u{869C}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{869D}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{86A3}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{86A4}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{86A7}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{86A8}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{86A9}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{86AA}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{86AC}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{86AF}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{86B0}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{86B1}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{86B2}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{86B4}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{86B6}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{86BA}',
        readings: &["ran"],
    },
    CharPinyinEntry {
        ch: '\u{86C0}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{86C3}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{86C4}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{86C6}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{86C7}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{86C9}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{86CA}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{86CB}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{86CE}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{86CF}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{86D0}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{86D1}',
        readings: &["mou"],
    },
    CharPinyinEntry {
        ch: '\u{86D4}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{86D8}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{86D9}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{86DB}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{86DE}',
        readings: &["kuo"],
    },
    CharPinyinEntry {
        ch: '\u{86DF}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{86E4}',
        readings: &["ge", "ha"],
    },
    CharPinyinEntry {
        ch: '\u{86E9}',
        readings: &["qiong"],
    },
    CharPinyinEntry {
        ch: '\u{86ED}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{86EE}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{86F0}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{86F1}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{86F2}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{86F3}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{86F4}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{86F8}',
        readings: &["shao", "xiao"],
    },
    CharPinyinEntry {
        ch: '\u{86F9}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{86FE}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{8700}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{8702}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{8703}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{8707}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{8708}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{8709}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{870A}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{870D}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{870E}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{8710}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{8712}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{8713}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{8715}',
        readings: &["tui"],
    },
    CharPinyinEntry {
        ch: '\u{8717}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{8718}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{871A}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{871C}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{871E}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{8721}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{8722}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{8723}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{8725}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{8729}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{872E}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8731}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{8734}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8737}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{873B}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{873E}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{873F}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{8747}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{8748}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{8749}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{874C}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{874E}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{8753}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8757}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{8758}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{8759}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{8760}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8763}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{8764}',
        readings: &["qiu", "you"],
    },
    CharPinyinEntry {
        ch: '\u{8765}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{876E}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8770}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{8772}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{8774}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{8776}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{877B}',
        readings: &["nan"],
    },
    CharPinyinEntry {
        ch: '\u{877C}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{877D}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{877E}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{8782}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{8783}',
        readings: &["pang"],
    },
    CharPinyinEntry {
        ch: '\u{8785}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{8788}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{878B}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{878D}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{8797}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{879F}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{87A0}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{87A3}',
        readings: &["teng"],
    },
    CharPinyinEntry {
        ch: '\u{87A8}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{87AB}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{87AC}',
        readings: &["cao"],
    },
    CharPinyinEntry {
        ch: '\u{87AD}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{87AF}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{87B1}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{87B3}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{87B5}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{87BA}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{87BD}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{87C0}',
        readings: &["shuai"],
    },
    CharPinyinEntry {
        ch: '\u{87C6}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{87CA}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{87CB}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{87CF}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{87D1}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{87D2}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{87DB}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{87E0}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{87E5}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{87EA}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{87EB}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{87EE}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{87F9}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{87FE}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{8803}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{880A}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{880B}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{8813}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{8815}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{8816}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{8821}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{8822}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{8832}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{8839}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{883C}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{8840}',
        readings: &["xie", "xue"],
    },
    CharPinyinEntry {
        ch: '\u{8843}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{8844}',
        readings: &["nv"],
    },
    CharPinyinEntry {
        ch: '\u{8845}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{884C}',
        readings: &["hang", "heng", "xing"],
    },
    CharPinyinEntry {
        ch: '\u{884D}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{884E}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{8852}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{8854}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{8857}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{8859}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{8860}',
        readings: &["zhun"],
    },
    CharPinyinEntry {
        ch: '\u{8861}',
        readings: &["heng"],
    },
    CharPinyinEntry {
        ch: '\u{8862}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{8863}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8865}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{8868}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{8869}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{886B}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{886C}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{886E}',
        readings: &["gun"],
    },
    CharPinyinEntry {
        ch: '\u{8870}',
        readings: &["shuai"],
    },
    CharPinyinEntry {
        ch: '\u{8872}',
        readings: &["na"],
    },
    CharPinyinEntry {
        ch: '\u{8877}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{887D}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{887E}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{887F}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{8881}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{8882}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{8884}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{8885}',
        readings: &["niao"],
    },
    CharPinyinEntry {
        ch: '\u{8886}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{8888}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{888B}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{888D}',
        readings: &["pao"],
    },
    CharPinyinEntry {
        ch: '\u{8892}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{8896}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{8897}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{889C}',
        readings: &["wa"],
    },
    CharPinyinEntry {
        ch: '\u{88A2}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{88A4}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{88AA}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{88AB}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{88AD}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{88AF}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{88B1}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{88B7}',
        readings: &["qia"],
    },
    CharPinyinEntry {
        ch: '\u{88BC}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{88C1}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{88C2}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{88C5}',
        readings: &["zhuang"],
    },
    CharPinyinEntry {
        ch: '\u{88C6}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{88C8}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{88C9}',
        readings: &["ken"],
    },
    CharPinyinEntry {
        ch: '\u{88CE}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{88D2}',
        readings: &["pou"],
    },
    CharPinyinEntry {
        ch: '\u{88D4}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{88D5}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{88D8}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{88D9}',
        readings: &["qun"],
    },
    CharPinyinEntry {
        ch: '\u{88DB}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{88DF}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{88E2}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{88E3}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{88E4}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{88E5}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{88E8}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{88F0}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{88F1}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{88F3}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{88F4}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{88F8}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{88F9}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{88FC}',
        readings: &["ti", "xi"],
    },
    CharPinyinEntry {
        ch: '\u{88FE}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{8902}',
        readings: &["gua"],
    },
    CharPinyinEntry {
        ch: '\u{890A}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{8910}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{8912}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{8913}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{8915}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8919}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{891A}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{891B}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{891F}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{8921}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{8925}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{892A}',
        readings: &["tui", "tun"],
    },
    CharPinyinEntry {
        ch: '\u{892B}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{892F}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{8930}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{8934}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{8936}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{8941}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{8944}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{8955}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{895A}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{895C}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{895E}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{895F}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{8966}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{896B}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{897B}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{897F}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{8981}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{8983}',
        readings: &["qin", "tan"],
    },
    CharPinyinEntry {
        ch: '\u{8986}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{89C1}',
        readings: &["jian", "xian"],
    },
    CharPinyinEntry {
        ch: '\u{89C2}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{89C3}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{89C4}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{89C5}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{89C6}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{89C7}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{89C8}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{89C9}',
        readings: &["jiao", "jue"],
    },
    CharPinyinEntry {
        ch: '\u{89CA}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{89CB}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{89CC}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{89CE}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{89CF}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{89D0}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{89D1}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{89D2}',
        readings: &["jiao", "jue"],
    },
    CharPinyinEntry {
        ch: '\u{89D6}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{89DA}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{89DC}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{89DE}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{89DF}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{89E3}',
        readings: &["jie", "xie"],
    },
    CharPinyinEntry {
        ch: '\u{89E5}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{89E6}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{89EB}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{89ED}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{89EF}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{89F1}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{89F3}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{89FF}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{8A00}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{8A04}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{8A07}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{8A1A}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{8A3E}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{8A48}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{8A5F}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{8A79}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{8A89}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8A8A}',
        readings: &["teng"],
    },
    CharPinyinEntry {
        ch: '\u{8A93}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{8B07}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8B66}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{8B6C}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{8BA1}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8BA2}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{8BA3}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8BA4}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{8BA5}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8BA6}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{8BA7}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{8BA8}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{8BA9}',
        readings: &["rang"],
    },
    CharPinyinEntry {
        ch: '\u{8BAA}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{8BAB}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{8BAD}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{8BAE}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8BAF}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{8BB0}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8BB1}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{8BB2}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{8BB3}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{8BB4}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{8BB5}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{8BB6}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{8BB7}',
        readings: &["ne"],
    },
    CharPinyinEntry {
        ch: '\u{8BB8}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{8BB9}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{8BBA}',
        readings: &["lun"],
    },
    CharPinyinEntry {
        ch: '\u{8BBB}',
        readings: &["xiong"],
    },
    CharPinyinEntry {
        ch: '\u{8BBC}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{8BBD}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{8BBE}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{8BBF}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{8BC0}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{8BC1}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{8BC2}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{8BC3}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{8BC4}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{8BC5}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{8BC6}',
        readings: &["shi", "zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8BC7}',
        readings: &["xiong"],
    },
    CharPinyinEntry {
        ch: '\u{8BC8}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{8BC9}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{8BCA}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{8BCB}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{8BCC}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{8BCD}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{8BCE}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{8BCF}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{8BD0}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{8BD1}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8BD2}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8BD3}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{8BD4}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{8BD5}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{8BD6}',
        readings: &["gua"],
    },
    CharPinyinEntry {
        ch: '\u{8BD7}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{8BD8}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{8BD9}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{8BDA}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{8BDB}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{8BDC}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{8BDD}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{8BDE}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{8BDF}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{8BE0}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{8BE1}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{8BE2}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{8BE3}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8BE4}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{8BE5}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{8BE6}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{8BE7}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{8BE8}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{8BE9}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{8BEB}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{8BEC}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{8BED}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8BEE}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{8BEF}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{8BF0}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{8BF1}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{8BF2}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{8BF3}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{8BF4}',
        readings: &["shui", "shuo"],
    },
    CharPinyinEntry {
        ch: '\u{8BF5}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{8BF7}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{8BF8}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{8BF9}',
        readings: &["zou"],
    },
    CharPinyinEntry {
        ch: '\u{8BFA}',
        readings: &["nuo"],
    },
    CharPinyinEntry {
        ch: '\u{8BFB}',
        readings: &["dou", "du"],
    },
    CharPinyinEntry {
        ch: '\u{8BFC}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{8BFD}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{8BFE}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{8BFF}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{8C00}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8C01}',
        readings: &["shei", "shui"],
    },
    CharPinyinEntry {
        ch: '\u{8C02}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{8C03}',
        readings: &["diao", "tiao"],
    },
    CharPinyinEntry {
        ch: '\u{8C04}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{8C05}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{8C06}',
        readings: &["zhun"],
    },
    CharPinyinEntry {
        ch: '\u{8C07}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{8C08}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{8C0A}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8C0B}',
        readings: &["mou"],
    },
    CharPinyinEntry {
        ch: '\u{8C0C}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{8C0D}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{8C0E}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{8C0F}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8C10}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{8C11}',
        readings: &["xue"],
    },
    CharPinyinEntry {
        ch: '\u{8C12}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{8C13}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{8C14}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{8C15}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8C16}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{8C17}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{8C19}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{8C1A}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{8C1B}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{8C1C}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{8C1D}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{8C1E}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{8C1F}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{8C20}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{8C21}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{8C22}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{8C23}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{8C24}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{8C25}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{8C26}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{8C27}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{8C28}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{8C29}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{8C2A}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{8C2B}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8C2C}',
        readings: &["miu"],
    },
    CharPinyinEntry {
        ch: '\u{8C2D}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{8C2E}',
        readings: &["zen"],
    },
    CharPinyinEntry {
        ch: '\u{8C2F}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{8C30}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{8C31}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{8C32}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{8C33}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{8C34}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{8C35}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{8C36}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{8C37}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{8C3C}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{8C3F}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{8C41}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{8C46}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{8C47}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{8C49}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{8C4C}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{8C55}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{8C5A}',
        readings: &["tun"],
    },
    CharPinyinEntry {
        ch: '\u{8C61}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{8C62}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{8C68}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{8C6A}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{8C6B}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8C6E}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{8C73}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{8C78}',
        readings: &["zhai", "zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8C79}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{8C7A}',
        readings: &["chai"],
    },
    CharPinyinEntry {
        ch: '\u{8C82}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{8C85}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{8C86}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{8C89}',
        readings: &["hao", "he"],
    },
    CharPinyinEntry {
        ch: '\u{8C8A}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{8C8C}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{8C94}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{8C98}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{8D1D}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{8D1E}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{8D1F}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8D21}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{8D22}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{8D23}',
        readings: &["ze"],
    },
    CharPinyinEntry {
        ch: '\u{8D24}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{8D25}',
        readings: &["bai"],
    },
    CharPinyinEntry {
        ch: '\u{8D26}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{8D27}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{8D28}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8D29}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{8D2A}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{8D2B}',
        readings: &["pin"],
    },
    CharPinyinEntry {
        ch: '\u{8D2C}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{8D2D}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{8D2E}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{8D2F}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{8D30}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{8D31}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8D32}',
        readings: &["ben"],
    },
    CharPinyinEntry {
        ch: '\u{8D33}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{8D34}',
        readings: &["tie"],
    },
    CharPinyinEntry {
        ch: '\u{8D35}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{8D36}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{8D37}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{8D38}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{8D39}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{8D3A}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{8D3B}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8D3C}',
        readings: &["zei"],
    },
    CharPinyinEntry {
        ch: '\u{8D3D}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8D3E}',
        readings: &["gu", "jia"],
    },
    CharPinyinEntry {
        ch: '\u{8D3F}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{8D40}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{8D41}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{8D42}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{8D43}',
        readings: &["zang"],
    },
    CharPinyinEntry {
        ch: '\u{8D44}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{8D45}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{8D46}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{8D47}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{8D48}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{8D49}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{8D4A}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{8D4B}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8D4C}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{8D4D}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8D4E}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{8D4F}',
        readings: &["shang"],
    },
    CharPinyinEntry {
        ch: '\u{8D50}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{8D51}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{8D52}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{8D53}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{8D54}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{8D55}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{8D56}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{8D57}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{8D58}',
        readings: &["zhui"],
    },
    CharPinyinEntry {
        ch: '\u{8D59}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8D5A}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{8D5B}',
        readings: &["sai"],
    },
    CharPinyinEntry {
        ch: '\u{8D5C}',
        readings: &["ze"],
    },
    CharPinyinEntry {
        ch: '\u{8D5D}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{8D5E}',
        readings: &["zan"],
    },
    CharPinyinEntry {
        ch: '\u{8D5F}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{8D60}',
        readings: &["zeng"],
    },
    CharPinyinEntry {
        ch: '\u{8D61}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{8D62}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{8D63}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{8D64}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{8D66}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{8D67}',
        readings: &["nan"],
    },
    CharPinyinEntry {
        ch: '\u{8D6A}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{8D6B}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{8D6D}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{8D70}',
        readings: &["zou"],
    },
    CharPinyinEntry {
        ch: '\u{8D73}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{8D74}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8D75}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{8D76}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{8D77}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{8D81}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{8D84}',
        readings: &["ju", "qie"],
    },
    CharPinyinEntry {
        ch: '\u{8D85}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{8D8A}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{8D8B}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{8D91}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{8D94}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{8D9F}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{8DA3}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{8DAF}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{8DB1}',
        readings: &["zan"],
    },
    CharPinyinEntry {
        ch: '\u{8DB3}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{8DB4}',
        readings: &["pa"],
    },
    CharPinyinEntry {
        ch: '\u{8DB5}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{8DB8}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{8DBA}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8DBC}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8DBE}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8DBF}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{8DC2}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{8DC3}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{8DC4}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{8DC6}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{8DCB}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{8DCC}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{8DCE}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{8DCF}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{8DD0}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{8DD1}',
        readings: &["pao"],
    },
    CharPinyinEntry {
        ch: '\u{8DD6}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8DD7}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8DDA}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{8DDB}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{8DDD}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{8DDE}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{8DDF}',
        readings: &["gen"],
    },
    CharPinyinEntry {
        ch: '\u{8DE3}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{8DE4}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{8DE8}',
        readings: &["kua"],
    },
    CharPinyinEntry {
        ch: '\u{8DEA}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{8DEC}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{8DEF}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{8DF1}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8DF3}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{8DF5}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8DF6}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{8DF7}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{8DF8}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{8DF9}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{8DFA}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{8DFB}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8DFD}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8E05}',
        readings: &["xue"],
    },
    CharPinyinEntry {
        ch: '\u{8E09}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{8E0A}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{8E0C}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{8E0F}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{8E12}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{8E14}',
        readings: &["chuo"],
    },
    CharPinyinEntry {
        ch: '\u{8E1D}',
        readings: &["huai"],
    },
    CharPinyinEntry {
        ch: '\u{8E1E}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{8E1F}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{8E22}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{8E23}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{8E26}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8E29}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{8E2A}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{8E2C}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8E2E}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{8E2F}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8E31}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{8E35}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{8E36}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{8E39}',
        readings: &["chuai"],
    },
    CharPinyinEntry {
        ch: '\u{8E3A}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8E3D}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{8E40}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{8E41}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{8E42}',
        readings: &["rou"],
    },
    CharPinyinEntry {
        ch: '\u{8E44}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{8E45}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{8E47}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{8E48}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{8E49}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{8E4A}',
        readings: &["qi", "xi"],
    },
    CharPinyinEntry {
        ch: '\u{8E4B}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{8E50}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8E51}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{8E52}',
        readings: &["pan"],
    },
    CharPinyinEntry {
        ch: '\u{8E59}',
        readings: &["cu"],
    },
    CharPinyinEntry {
        ch: '\u{8E5A}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{8E5C}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{8E62}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{8E66}',
        readings: &["beng"],
    },
    CharPinyinEntry {
        ch: '\u{8E69}',
        readings: &["bie"],
    },
    CharPinyinEntry {
        ch: '\u{8E6C}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{8E6D}',
        readings: &["ceng"],
    },
    CharPinyinEntry {
        ch: '\u{8E6F}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{8E70}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{8E72}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{8E74}',
        readings: &["cu"],
    },
    CharPinyinEntry {
        ch: '\u{8E76}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{8E7C}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{8E7D}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{8E7E}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{8E7F}',
        readings: &["cuan"],
    },
    CharPinyinEntry {
        ch: '\u{8E81}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{8E85}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{8E87}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{8E8F}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{8E90}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{8E94}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{8E9C}',
        readings: &["zuan"],
    },
    CharPinyinEntry {
        ch: '\u{8E9E}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{8EAB}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{8EAC}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{8EAF}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{8EB2}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{8EBA}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{8F66}',
        readings: &["che", "ju"],
    },
    CharPinyinEntry {
        ch: '\u{8F67}',
        readings: &["ya", "zha"],
    },
    CharPinyinEntry {
        ch: '\u{8F68}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{8F69}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{8F6A}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{8F6B}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{8F6C}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{8F6D}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{8F6E}',
        readings: &["lun"],
    },
    CharPinyinEntry {
        ch: '\u{8F6F}',
        readings: &["ruan"],
    },
    CharPinyinEntry {
        ch: '\u{8F70}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{8F71}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{8F72}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{8F73}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{8F74}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{8F75}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8F76}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8F77}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{8F78}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{8F79}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{8F7A}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{8F7B}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{8F7C}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{8F7D}',
        readings: &["zai"],
    },
    CharPinyinEntry {
        ch: '\u{8F7E}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{8F7F}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{8F80}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{8F81}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{8F82}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{8F83}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{8F84}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{8F85}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8F86}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{8F87}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{8F88}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{8F89}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{8F8A}',
        readings: &["gun"],
    },
    CharPinyinEntry {
        ch: '\u{8F8B}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{8F8C}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{8F8D}',
        readings: &["chuo"],
    },
    CharPinyinEntry {
        ch: '\u{8F8E}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{8F8F}',
        readings: &["cou"],
    },
    CharPinyinEntry {
        ch: '\u{8F90}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{8F91}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8F92}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{8F93}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{8F94}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{8F95}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{8F96}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{8F97}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{8F98}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{8F99}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{8F9A}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{8F9B}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{8F9C}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{8F9E}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{8F9F}',
        readings: &["bi", "pi"],
    },
    CharPinyinEntry {
        ch: '\u{8FA3}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{8FA8}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{8FA9}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{8FAB}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{8FB0}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{8FB1}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{8FB9}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{8FBD}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{8FBE}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{8FBF}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{8FC1}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{8FC2}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{8FC4}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{8FC5}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{8FC7}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{8FC8}',
        readings: &["mai"],
    },
    CharPinyinEntry {
        ch: '\u{8FCE}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{8FD0}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{8FD1}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{8FD3}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{8FD4}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{8FD5}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{8FD8}',
        readings: &["hai", "huan"],
    },
    CharPinyinEntry {
        ch: '\u{8FD9}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{8FDB}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{8FDC}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{8FDD}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{8FDE}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{8FDF}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{8FE2}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{8FE4}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{8FE5}',
        readings: &["jiong"],
    },
    CharPinyinEntry {
        ch: '\u{8FE6}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{8FE8}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{8FE9}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{8FEA}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{8FEB}',
        readings: &["pai", "po"],
    },
    CharPinyinEntry {
        ch: '\u{8FED}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{8FEE}',
        readings: &["ze"],
    },
    CharPinyinEntry {
        ch: '\u{8FF0}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{8FF3}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{8FF7}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{8FF8}',
        readings: &["beng"],
    },
    CharPinyinEntry {
        ch: '\u{8FF9}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{8FFA}',
        readings: &["nai"],
    },
    CharPinyinEntry {
        ch: '\u{8FFD}',
        readings: &["zhui"],
    },
    CharPinyinEntry {
        ch: '\u{9000}',
        readings: &["tui"],
    },
    CharPinyinEntry {
        ch: '\u{9001}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{9002}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{9003}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{9004}',
        readings: &["pang"],
    },
    CharPinyinEntry {
        ch: '\u{9005}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{9006}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{9009}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{900A}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{900B}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{900D}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{900F}',
        readings: &["tou"],
    },
    CharPinyinEntry {
        ch: '\u{9010}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{9011}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{9012}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{9014}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{9016}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{9017}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{901A}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{901B}',
        readings: &["guang"],
    },
    CharPinyinEntry {
        ch: '\u{901D}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{901E}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{901F}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{9020}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{9021}',
        readings: &["qun"],
    },
    CharPinyinEntry {
        ch: '\u{9022}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{9026}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{902D}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{902E}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{902F}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{9034}',
        readings: &["chuo"],
    },
    CharPinyinEntry {
        ch: '\u{9035}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{9036}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{9038}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{903B}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{903C}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{903E}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9041}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{9042}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{9044}',
        readings: &["chuan"],
    },
    CharPinyinEntry {
        ch: '\u{9046}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{9047}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{904D}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{904F}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{9050}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{9051}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{9052}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{9053}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{9057}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9058}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{905B}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{9062}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{9063}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{9065}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{9068}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{906D}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{906E}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{9074}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{9075}',
        readings: &["zun"],
    },
    CharPinyinEntry {
        ch: '\u{9079}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{907D}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{907F}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{9080}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{9082}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{9083}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{9088}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{908B}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{9091}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9093}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{9095}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{9097}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{9098}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9099}',
        readings: &["mang"],
    },
    CharPinyinEntry {
        ch: '\u{909B}',
        readings: &["qiong"],
    },
    CharPinyinEntry {
        ch: '\u{909D}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{90A0}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{90A1}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{90A2}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{90A3}',
        readings: &["na"],
    },
    CharPinyinEntry {
        ch: '\u{90A6}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{90A8}',
        readings: &["cun"],
    },
    CharPinyinEntry {
        ch: '\u{90AA}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{90AC}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{90AE}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{90AF}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{90B0}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{90B1}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{90B2}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{90B3}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{90B4}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{90B5}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{90B6}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{90B8}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{90B9}',
        readings: &["zou"],
    },
    CharPinyinEntry {
        ch: '\u{90BA}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{90BB}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{90BD}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{90BE}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{90BF}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{90C1}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{90C3}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{90C4}',
        readings: &["qie"],
    },
    CharPinyinEntry {
        ch: '\u{90C5}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{90C7}',
        readings: &["huan", "xun"],
    },
    CharPinyinEntry {
        ch: '\u{90C8}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{90CA}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{90CE}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{90CF}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{90D0}',
        readings: &["kuai"],
    },
    CharPinyinEntry {
        ch: '\u{90D1}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{90D3}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{90D7}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{90DA}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{90DB}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{90DC}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{90DD}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{90E1}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{90E2}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{90E4}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{90E6}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{90E7}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{90E8}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{90EA}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{90EB}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{90ED}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{90EF}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{90F4}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{90F8}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{90FD}',
        readings: &["dou", "du"],
    },
    CharPinyinEntry {
        ch: '\u{90FE}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{90FF}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{9100}',
        readings: &["ruo"],
    },
    CharPinyinEntry {
        ch: '\u{9102}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{9103}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{9104}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{9105}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{910C}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{9111}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{9117}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{9118}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{9119}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{911A}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{911C}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{911E}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{9120}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{9122}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{9123}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{912B}',
        readings: &["zeng"],
    },
    CharPinyinEntry {
        ch: '\u{912F}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{9131}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{9139}',
        readings: &["zou"],
    },
    CharPinyinEntry {
        ch: '\u{9142}',
        readings: &["zan"],
    },
    CharPinyinEntry {
        ch: '\u{9143}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{9145}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{9146}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{9149}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{914A}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{914B}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{914C}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{914D}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{914E}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{914F}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9150}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{9152}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{9157}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{915A}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{915D}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{915E}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{9161}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{9162}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{9163}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{9164}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{9165}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{9166}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{9169}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{916A}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{916C}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{916E}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{916F}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{9170}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{9171}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{9172}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{9174}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{9175}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{9176}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{9177}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{9178}',
        readings: &["suan"],
    },
    CharPinyinEntry {
        ch: '\u{9179}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{917A}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{917D}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{917E}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{917F}',
        readings: &["niang"],
    },
    CharPinyinEntry {
        ch: '\u{9185}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{9187}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{9189}',
        readings: &["zui"],
    },
    CharPinyinEntry {
        ch: '\u{918B}',
        readings: &["cu"],
    },
    CharPinyinEntry {
        ch: '\u{918C}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{918D}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{9190}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{9191}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{9192}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{919A}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{919B}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{91A2}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{91A8}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{91AA}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{91AD}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{91AE}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{91AF}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{91B4}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{91B5}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{91BA}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{91BE}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{91C7}',
        readings: &["cai"],
    },
    CharPinyinEntry {
        ch: '\u{91C9}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{91CA}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{91CC}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{91CD}',
        readings: &["chong", "zhong"],
    },
    CharPinyinEntry {
        ch: '\u{91CE}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{91CF}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{91D0}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{91D1}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{91DC}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{9274}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{928E}',
        readings: &["qiong"],
    },
    CharPinyinEntry {
        ch: '\u{92AE}',
        readings: &["luan"],
    },
    CharPinyinEntry {
        ch: '\u{92C6}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{92C8}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{933E}',
        readings: &["zan"],
    },
    CharPinyinEntry {
        ch: '\u{936A}',
        readings: &["mou"],
    },
    CharPinyinEntry {
        ch: '\u{938F}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{93CA}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{93D6}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{943E}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{946B}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{9486}',
        readings: &["ga"],
    },
    CharPinyinEntry {
        ch: '\u{9487}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9488}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{9489}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{948A}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{948B}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{948C}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{948D}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{948E}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{948F}',
        readings: &["chuan"],
    },
    CharPinyinEntry {
        ch: '\u{9490}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{9492}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{9493}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{9494}',
        readings: &["men"],
    },
    CharPinyinEntry {
        ch: '\u{9495}',
        readings: &["nv"],
    },
    CharPinyinEntry {
        ch: '\u{9496}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{9497}',
        readings: &["chai"],
    },
    CharPinyinEntry {
        ch: '\u{9498}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{9499}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{949A}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{949B}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{949C}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{949D}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{949E}',
        readings: &["chao"],
    },
    CharPinyinEntry {
        ch: '\u{949F}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{94A0}',
        readings: &["na"],
    },
    CharPinyinEntry {
        ch: '\u{94A1}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{94A2}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{94A3}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{94A4}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{94A5}',
        readings: &["yao", "yue"],
    },
    CharPinyinEntry {
        ch: '\u{94A6}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{94A7}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{94A8}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{94A9}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{94AA}',
        readings: &["kang"],
    },
    CharPinyinEntry {
        ch: '\u{94AB}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{94AC}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{94AD}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{94AE}',
        readings: &["niu"],
    },
    CharPinyinEntry {
        ch: '\u{94AF}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{94B0}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{94B1}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{94B2}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{94B3}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{94B4}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{94B5}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{94B7}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{94B9}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{94BA}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{94BB}',
        readings: &["zuan"],
    },
    CharPinyinEntry {
        ch: '\u{94BC}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{94BD}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{94BE}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{94BF}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{94C0}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{94C1}',
        readings: &["tie"],
    },
    CharPinyinEntry {
        ch: '\u{94C2}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{94C3}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{94C4}',
        readings: &["shuo"],
    },
    CharPinyinEntry {
        ch: '\u{94C5}',
        readings: &["qian", "yan"],
    },
    CharPinyinEntry {
        ch: '\u{94C6}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{94C8}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{94C9}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{94CA}',
        readings: &["ta", "tuo"],
    },
    CharPinyinEntry {
        ch: '\u{94CB}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{94CC}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{94CD}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{94CE}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{94CF}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{94D0}',
        readings: &["kao"],
    },
    CharPinyinEntry {
        ch: '\u{94D1}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{94D2}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{94D5}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{94D6}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{94D7}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{94D8}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{94D9}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{94DA}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{94DB}',
        readings: &["cheng", "dang"],
    },
    CharPinyinEntry {
        ch: '\u{94DC}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{94DD}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{94DE}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{94DF}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{94E0}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{94E1}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{94E2}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{94E3}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{94E4}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{94E5}',
        readings: &["diu"],
    },
    CharPinyinEntry {
        ch: '\u{94E7}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{94E8}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{94E9}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{94EA}',
        readings: &["ha"],
    },
    CharPinyinEntry {
        ch: '\u{94EB}',
        readings: &["diao", "yao"],
    },
    CharPinyinEntry {
        ch: '\u{94EC}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{94ED}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{94EE}',
        readings: &["zheng"],
    },
    CharPinyinEntry {
        ch: '\u{94EF}',
        readings: &["se"],
    },
    CharPinyinEntry {
        ch: '\u{94F0}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{94F1}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{94F2}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{94F3}',
        readings: &["chong"],
    },
    CharPinyinEntry {
        ch: '\u{94F4}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{94F5}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{94F6}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{94F7}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{94F8}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{94F9}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{94FA}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{94FB}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{94FC}',
        readings: &["lai"],
    },
    CharPinyinEntry {
        ch: '\u{94FD}',
        readings: &["te"],
    },
    CharPinyinEntry {
        ch: '\u{94FE}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{94FF}',
        readings: &["keng"],
    },
    CharPinyinEntry {
        ch: '\u{9500}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{9501}',
        readings: &["suo"],
    },
    CharPinyinEntry {
        ch: '\u{9502}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{9503}',
        readings: &["zeng"],
    },
    CharPinyinEntry {
        ch: '\u{9504}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{9505}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{9506}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{9507}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{9508}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{9509}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{950A}',
        readings: &["lve"],
    },
    CharPinyinEntry {
        ch: '\u{950B}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{950C}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{950D}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{950E}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{950F}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{9510}',
        readings: &["rui"],
    },
    CharPinyinEntry {
        ch: '\u{9511}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{9512}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{9513}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{9514}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{9515}',
        readings: &["a"],
    },
    CharPinyinEntry {
        ch: '\u{9516}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{9517}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{9518}',
        readings: &["nuo"],
    },
    CharPinyinEntry {
        ch: '\u{9519}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{951A}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{951B}',
        readings: &["ben"],
    },
    CharPinyinEntry {
        ch: '\u{951C}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{951D}',
        readings: &["de"],
    },
    CharPinyinEntry {
        ch: '\u{951E}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{951F}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{9521}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{9522}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{9523}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{9524}',
        readings: &["chui"],
    },
    CharPinyinEntry {
        ch: '\u{9525}',
        readings: &["zhui"],
    },
    CharPinyinEntry {
        ch: '\u{9526}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{9527}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{9528}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{9529}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{952A}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{952B}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{952C}',
        readings: &["tan"],
    },
    CharPinyinEntry {
        ch: '\u{952D}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{952E}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{952F}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{9530}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{9531}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{9532}',
        readings: &["qie"],
    },
    CharPinyinEntry {
        ch: '\u{9533}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{9534}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{9535}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{9536}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{9537}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{9538}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{9539}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{953A}',
        readings: &["zhong"],
    },
    CharPinyinEntry {
        ch: '\u{953B}',
        readings: &["duan"],
    },
    CharPinyinEntry {
        ch: '\u{953C}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{953D}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{953E}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{953F}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{9540}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{9541}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{9542}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{9543}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{9544}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{9545}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{9546}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{9547}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{9548}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{9549}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{954A}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{954B}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{954C}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{954D}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{954E}',
        readings: &["na"],
    },
    CharPinyinEntry {
        ch: '\u{954F}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{9550}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{9551}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{9552}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9553}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{9554}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{9555}',
        readings: &["rong"],
    },
    CharPinyinEntry {
        ch: '\u{9556}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{9557}',
        readings: &["tang"],
    },
    CharPinyinEntry {
        ch: '\u{9558}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{955A}',
        readings: &["beng"],
    },
    CharPinyinEntry {
        ch: '\u{955B}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{955C}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{955D}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{955E}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{9560}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{9561}',
        readings: &["chan", "tan"],
    },
    CharPinyinEntry {
        ch: '\u{9562}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{9563}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{9564}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{9565}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{9566}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{9567}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{9568}',
        readings: &["pu"],
    },
    CharPinyinEntry {
        ch: '\u{9569}',
        readings: &["cuan"],
    },
    CharPinyinEntry {
        ch: '\u{956A}',
        readings: &["qiang"],
    },
    CharPinyinEntry {
        ch: '\u{956B}',
        readings: &["deng"],
    },
    CharPinyinEntry {
        ch: '\u{956C}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{956D}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{956E}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{956F}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{9570}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{9571}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9572}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{9573}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{9574}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{9575}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{9576}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{957F}',
        readings: &["chang", "zhang"],
    },
    CharPinyinEntry {
        ch: '\u{95E8}',
        readings: &["men"],
    },
    CharPinyinEntry {
        ch: '\u{95E9}',
        readings: &["shuan"],
    },
    CharPinyinEntry {
        ch: '\u{95EA}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{95EB}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{95ED}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{95EE}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{95EF}',
        readings: &["chuang"],
    },
    CharPinyinEntry {
        ch: '\u{95F0}',
        readings: &["run"],
    },
    CharPinyinEntry {
        ch: '\u{95F1}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{95F2}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{95F3}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{95F4}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{95F5}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{95F6}',
        readings: &["kang"],
    },
    CharPinyinEntry {
        ch: '\u{95F7}',
        readings: &["men"],
    },
    CharPinyinEntry {
        ch: '\u{95F8}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{95F9}',
        readings: &["nao"],
    },
    CharPinyinEntry {
        ch: '\u{95FA}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{95FB}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{95FC}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{95FD}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{95FE}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{95FF}',
        readings: &["kai"],
    },
    CharPinyinEntry {
        ch: '\u{9600}',
        readings: &["fa"],
    },
    CharPinyinEntry {
        ch: '\u{9601}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{9602}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{9603}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{9604}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{9605}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{9606}',
        readings: &["lang"],
    },
    CharPinyinEntry {
        ch: '\u{9607}',
        readings: &["du", "she"],
    },
    CharPinyinEntry {
        ch: '\u{9608}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9609}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{960A}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{960B}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{960C}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{960D}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{960E}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{960F}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{9610}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{9611}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{9612}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{9614}',
        readings: &["kuo"],
    },
    CharPinyinEntry {
        ch: '\u{9615}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{9616}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{9617}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{9618}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{9619}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{961A}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{961C}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{961F}',
        readings: &["dui"],
    },
    CharPinyinEntry {
        ch: '\u{9621}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{962A}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{962E}',
        readings: &["ruan"],
    },
    CharPinyinEntry {
        ch: '\u{9631}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{9632}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{9633}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{9634}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{9635}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{9636}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{963B}',
        readings: &["zu"],
    },
    CharPinyinEntry {
        ch: '\u{963C}',
        readings: &["zuo"],
    },
    CharPinyinEntry {
        ch: '\u{963D}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{963F}',
        readings: &["a", "e"],
    },
    CharPinyinEntry {
        ch: '\u{9640}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{9642}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{9644}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{9645}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9646}',
        readings: &["liu", "lu"],
    },
    CharPinyinEntry {
        ch: '\u{9647}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{9648}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{9649}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{964B}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{964C}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{964D}',
        readings: &["jiang", "xiang"],
    },
    CharPinyinEntry {
        ch: '\u{964E}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{9650}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{9651}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{9654}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{9655}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{965B}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{965E}',
        readings: &["sheng"],
    },
    CharPinyinEntry {
        ch: '\u{965F}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{9661}',
        readings: &["dou"],
    },
    CharPinyinEntry {
        ch: '\u{9662}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{9664}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{9667}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{9668}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{9669}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{966A}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{966C}',
        readings: &["zou"],
    },
    CharPinyinEntry {
        ch: '\u{9672}',
        readings: &["chui"],
    },
    CharPinyinEntry {
        ch: '\u{9674}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{9675}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{9676}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{9677}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{9683}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{9685}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9686}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{9688}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{968B}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{968D}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{968F}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{9690}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{9694}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{9697}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{9698}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{9699}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{969C}',
        readings: &["zhang"],
    },
    CharPinyinEntry {
        ch: '\u{96A7}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{96A9}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{96B0}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{96B3}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{96B6}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{96B9}',
        readings: &["zhui"],
    },
    CharPinyinEntry {
        ch: '\u{96BA}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{96BC}',
        readings: &["sun"],
    },
    CharPinyinEntry {
        ch: '\u{96BD}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{96BE}',
        readings: &["nan"],
    },
    CharPinyinEntry {
        ch: '\u{96C0}',
        readings: &["qiao", "que"],
    },
    CharPinyinEntry {
        ch: '\u{96C1}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{96C4}',
        readings: &["xiong"],
    },
    CharPinyinEntry {
        ch: '\u{96C5}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{96C6}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{96C7}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{96C9}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{96CA}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{96CC}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{96CD}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{96CE}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{96CF}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{96D2}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{96D5}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{96E0}',
        readings: &["chou"],
    },
    CharPinyinEntry {
        ch: '\u{96E8}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{96E9}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{96EA}',
        readings: &["xue"],
    },
    CharPinyinEntry {
        ch: '\u{96EF}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{96F1}',
        readings: &["pang"],
    },
    CharPinyinEntry {
        ch: '\u{96F3}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{96F6}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{96F7}',
        readings: &["lei"],
    },
    CharPinyinEntry {
        ch: '\u{96F9}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{96FE}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{9700}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{9701}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9704}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{9705}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{9706}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{9707}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{9708}',
        readings: &["pei"],
    },
    CharPinyinEntry {
        ch: '\u{9709}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{970D}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{970E}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{970F}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{9713}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{9716}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{971C}',
        readings: &["shuang"],
    },
    CharPinyinEntry {
        ch: '\u{971E}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{9728}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{972A}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{972D}',
        readings: &["ai"],
    },
    CharPinyinEntry {
        ch: '\u{9730}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{9732}',
        readings: &["lou", "lu"],
    },
    CharPinyinEntry {
        ch: '\u{9738}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{9739}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{973E}',
        readings: &["mai"],
    },
    CharPinyinEntry {
        ch: '\u{9752}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{9753}',
        readings: &["jing", "liang"],
    },
    CharPinyinEntry {
        ch: '\u{9756}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{9759}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{975B}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{975E}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{9760}',
        readings: &["kao"],
    },
    CharPinyinEntry {
        ch: '\u{9761}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{9762}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{9765}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{9769}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{976C}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{9770}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{9773}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{9774}',
        readings: &["xue"],
    },
    CharPinyinEntry {
        ch: '\u{9776}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{9778}',
        readings: &["sa"],
    },
    CharPinyinEntry {
        ch: '\u{977A}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{977C}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{977D}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{977F}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{9781}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{9785}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{978B}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{978D}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{9791}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{9792}',
        readings: &["qiao"],
    },
    CharPinyinEntry {
        ch: '\u{9794}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{9798}',
        readings: &["qiao", "shao"],
    },
    CharPinyinEntry {
        ch: '\u{97A0}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{97A1}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{97A3}',
        readings: &["rou"],
    },
    CharPinyinEntry {
        ch: '\u{97A7}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{97A8}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{97AB}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{97AC}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{97AD}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{97AE}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{97AF}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{97B2}',
        readings: &["gou"],
    },
    CharPinyinEntry {
        ch: '\u{97B3}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{97B4}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{97C2}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{97E6}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{97E7}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{97E8}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{97E9}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{97EA}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{97EB}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{97EC}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{97ED}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{97F3}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{97F5}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{97F6}',
        readings: &["shao"],
    },
    CharPinyinEntry {
        ch: '\u{9875}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{9876}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{9877}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{9878}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{9879}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{987A}',
        readings: &["shun"],
    },
    CharPinyinEntry {
        ch: '\u{987B}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{987C}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{987D}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{987E}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{987F}',
        readings: &["dun"],
    },
    CharPinyinEntry {
        ch: '\u{9880}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{9881}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{9882}',
        readings: &["song"],
    },
    CharPinyinEntry {
        ch: '\u{9883}',
        readings: &["hang"],
    },
    CharPinyinEntry {
        ch: '\u{9884}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9885}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{9886}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{9887}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{9888}',
        readings: &["geng", "jing"],
    },
    CharPinyinEntry {
        ch: '\u{9889}',
        readings: &["jie", "xie"],
    },
    CharPinyinEntry {
        ch: '\u{988A}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{988B}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{988C}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{988D}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{988E}',
        readings: &["jiong"],
    },
    CharPinyinEntry {
        ch: '\u{988F}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{9890}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9891}',
        readings: &["pin"],
    },
    CharPinyinEntry {
        ch: '\u{9893}',
        readings: &["tui"],
    },
    CharPinyinEntry {
        ch: '\u{9894}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{9896}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{9897}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{9898}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{9899}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{989A}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{989B}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{989C}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{989D}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{989E}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{989F}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{98A0}',
        readings: &["dian"],
    },
    CharPinyinEntry {
        ch: '\u{98A1}',
        readings: &["sang"],
    },
    CharPinyinEntry {
        ch: '\u{98A2}',
        readings: &["hao"],
    },
    CharPinyinEntry {
        ch: '\u{98A4}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{98A5}',
        readings: &["ru"],
    },
    CharPinyinEntry {
        ch: '\u{98A6}',
        readings: &["pin"],
    },
    CharPinyinEntry {
        ch: '\u{98A7}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{98CE}',
        readings: &["feng"],
    },
    CharPinyinEntry {
        ch: '\u{98CF}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{98D0}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{98D1}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{98D2}',
        readings: &["sa"],
    },
    CharPinyinEntry {
        ch: '\u{98D3}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{98D4}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{98D5}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{98D7}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{98D8}',
        readings: &["piao"],
    },
    CharPinyinEntry {
        ch: '\u{98D9}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{98DE}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{98DF}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{98E7}',
        readings: &["sun"],
    },
    CharPinyinEntry {
        ch: '\u{98E8}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{990D}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{9910}',
        readings: &["can"],
    },
    CharPinyinEntry {
        ch: '\u{992E}',
        readings: &["tie"],
    },
    CharPinyinEntry {
        ch: '\u{9954}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{9955}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{9965}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9967}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{9968}',
        readings: &["tun"],
    },
    CharPinyinEntry {
        ch: '\u{9969}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{996A}',
        readings: &["ren"],
    },
    CharPinyinEntry {
        ch: '\u{996B}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{996C}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{996D}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{996E}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{996F}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{9970}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{9971}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{9972}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{9973}',
        readings: &["duo"],
    },
    CharPinyinEntry {
        ch: '\u{9974}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9975}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{9976}',
        readings: &["rao"],
    },
    CharPinyinEntry {
        ch: '\u{9977}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{9978}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{9979}',
        readings: &["ge", "le"],
    },
    CharPinyinEntry {
        ch: '\u{997A}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{997B}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{997C}',
        readings: &["bing"],
    },
    CharPinyinEntry {
        ch: '\u{997D}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{997F}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{9981}',
        readings: &["nei"],
    },
    CharPinyinEntry {
        ch: '\u{9983}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{9984}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{9985}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{9986}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{9987}',
        readings: &["cha", "zha"],
    },
    CharPinyinEntry {
        ch: '\u{9988}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{9989}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{998A}',
        readings: &["sou"],
    },
    CharPinyinEntry {
        ch: '\u{998B}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{998C}',
        readings: &["ye"],
    },
    CharPinyinEntry {
        ch: '\u{998D}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{998F}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{9990}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{9991}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{9992}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{9993}',
        readings: &["san"],
    },
    CharPinyinEntry {
        ch: '\u{9994}',
        readings: &["zhuan"],
    },
    CharPinyinEntry {
        ch: '\u{9995}',
        readings: &["nang"],
    },
    CharPinyinEntry {
        ch: '\u{9996}',
        readings: &["shou"],
    },
    CharPinyinEntry {
        ch: '\u{9997}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{9998}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{9999}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{999D}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{999E}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{99A5}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{99A7}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{99A8}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{9A6C}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{9A6D}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9A6E}',
        readings: &["duo", "tuo"],
    },
    CharPinyinEntry {
        ch: '\u{9A6F}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{9A70}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{9A71}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{9A72}',
        readings: &["ri"],
    },
    CharPinyinEntry {
        ch: '\u{9A73}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{9A74}',
        readings: &["lv"],
    },
    CharPinyinEntry {
        ch: '\u{9A75}',
        readings: &["zang"],
    },
    CharPinyinEntry {
        ch: '\u{9A76}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{9A77}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{9A78}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{9A79}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{9A7A}',
        readings: &["zou"],
    },
    CharPinyinEntry {
        ch: '\u{9A7B}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{9A7C}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{9A7D}',
        readings: &["nu"],
    },
    CharPinyinEntry {
        ch: '\u{9A7E}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{9A7F}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9A80}',
        readings: &["dai", "tai"],
    },
    CharPinyinEntry {
        ch: '\u{9A81}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{9A82}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{9A83}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{9A84}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{9A85}',
        readings: &["hua"],
    },
    CharPinyinEntry {
        ch: '\u{9A86}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{9A87}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{9A88}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{9A89}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{9A8A}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{9A8B}',
        readings: &["cheng"],
    },
    CharPinyinEntry {
        ch: '\u{9A8C}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{9A8D}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{9A8E}',
        readings: &["qin"],
    },
    CharPinyinEntry {
        ch: '\u{9A8F}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{9A90}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{9A91}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{9A92}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{9A93}',
        readings: &["zhui"],
    },
    CharPinyinEntry {
        ch: '\u{9A95}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{9A96}',
        readings: &["can"],
    },
    CharPinyinEntry {
        ch: '\u{9A97}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{9A98}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{9A99}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{9A9A}',
        readings: &["sao"],
    },
    CharPinyinEntry {
        ch: '\u{9A9B}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{9A9C}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{9A9D}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{9A9E}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{9A9F}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{9AA0}',
        readings: &["biao", "piao"],
    },
    CharPinyinEntry {
        ch: '\u{9AA1}',
        readings: &["luo"],
    },
    CharPinyinEntry {
        ch: '\u{9AA2}',
        readings: &["cong"],
    },
    CharPinyinEntry {
        ch: '\u{9AA3}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{9AA4}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{9AA5}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9AA6}',
        readings: &["shuang"],
    },
    CharPinyinEntry {
        ch: '\u{9AA7}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{9AA8}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{9AB0}',
        readings: &["tou"],
    },
    CharPinyinEntry {
        ch: '\u{9AB1}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{9AB6}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{9AB7}',
        readings: &["ku"],
    },
    CharPinyinEntry {
        ch: '\u{9AB8}',
        readings: &["hai"],
    },
    CharPinyinEntry {
        ch: '\u{9ABA}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{9ABC}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{9AC0}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{9AC1}',
        readings: &["ke"],
    },
    CharPinyinEntry {
        ch: '\u{9AC2}',
        readings: &["qia"],
    },
    CharPinyinEntry {
        ch: '\u{9AC3}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9AC5}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{9ACB}',
        readings: &["kuan"],
    },
    CharPinyinEntry {
        ch: '\u{9ACC}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{9ACE}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{9AD1}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{9AD3}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{9AD8}',
        readings: &["gao"],
    },
    CharPinyinEntry {
        ch: '\u{9AE1}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{9AE2}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{9AE6}',
        readings: &["mao"],
    },
    CharPinyinEntry {
        ch: '\u{9AEB}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{9AED}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{9AEF}',
        readings: &["ran"],
    },
    CharPinyinEntry {
        ch: '\u{9AF9}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{9AFB}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9AFD}',
        readings: &["zhua"],
    },
    CharPinyinEntry {
        ch: '\u{9B03}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{9B08}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{9B0F}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{9B12}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{9B13}',
        readings: &["bin"],
    },
    CharPinyinEntry {
        ch: '\u{9B18}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{9B1F}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{9B23}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{9B2F}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{9B32}',
        readings: &["ge", "li"],
    },
    CharPinyinEntry {
        ch: '\u{9B36}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{9B37}',
        readings: &["zong"],
    },
    CharPinyinEntry {
        ch: '\u{9B3B}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9B3C}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{9B41}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{9B42}',
        readings: &["hun"],
    },
    CharPinyinEntry {
        ch: '\u{9B43}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{9B44}',
        readings: &["po"],
    },
    CharPinyinEntry {
        ch: '\u{9B45}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{9B46}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{9B47}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{9B48}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{9B49}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{9B4B}',
        readings: &["tui"],
    },
    CharPinyinEntry {
        ch: '\u{9B4D}',
        readings: &["wang"],
    },
    CharPinyinEntry {
        ch: '\u{9B4F}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{9B51}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{9B54}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{9C7C}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9C7D}',
        readings: &["dao"],
    },
    CharPinyinEntry {
        ch: '\u{9C7E}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9C7F}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{9C80}',
        readings: &["tun"],
    },
    CharPinyinEntry {
        ch: '\u{9C81}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{9C82}',
        readings: &["fang"],
    },
    CharPinyinEntry {
        ch: '\u{9C83}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{9C85}',
        readings: &["ba"],
    },
    CharPinyinEntry {
        ch: '\u{9C86}',
        readings: &["ping"],
    },
    CharPinyinEntry {
        ch: '\u{9C87}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{9C88}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{9C89}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{9C8A}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{9C8B}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{9C8C}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{9C8D}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{9C8E}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{9C8F}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{9C90}',
        readings: &["tai"],
    },
    CharPinyinEntry {
        ch: '\u{9C91}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{9C92}',
        readings: &["jie"],
    },
    CharPinyinEntry {
        ch: '\u{9C94}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{9C95}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{9C96}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{9C97}',
        readings: &["zei"],
    },
    CharPinyinEntry {
        ch: '\u{9C98}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{9C99}',
        readings: &["kuai"],
    },
    CharPinyinEntry {
        ch: '\u{9C9A}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9C9B}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{9C9C}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{9C9D}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{9C9E}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{9C9F}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{9CA0}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{9CA1}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{9CA2}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{9CA3}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{9CA4}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{9CA5}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{9CA6}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{9CA7}',
        readings: &["gun"],
    },
    CharPinyinEntry {
        ch: '\u{9CA8}',
        readings: &["sha"],
    },
    CharPinyinEntry {
        ch: '\u{9CA9}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{9CAA}',
        readings: &["jun"],
    },
    CharPinyinEntry {
        ch: '\u{9CAB}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9CAC}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{9CAD}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{9CAE}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{9CAF}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{9CB0}',
        readings: &["zou"],
    },
    CharPinyinEntry {
        ch: '\u{9CB1}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{9CB2}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{9CB3}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{9CB4}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{9CB5}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{9CB7}',
        readings: &["diao"],
    },
    CharPinyinEntry {
        ch: '\u{9CB8}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{9CB9}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{9CBA}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{9CBB}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{9CBC}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{9CBD}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{9CBE}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{9CBF}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{9CC0}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{9CC1}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{9CC2}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{9CC3}',
        readings: &["sai"],
    },
    CharPinyinEntry {
        ch: '\u{9CC4}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{9CC5}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{9CC7}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{9CC8}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{9CC9}',
        readings: &["jiang"],
    },
    CharPinyinEntry {
        ch: '\u{9CCA}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{9CCC}',
        readings: &["ao"],
    },
    CharPinyinEntry {
        ch: '\u{9CCD}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{9CCE}',
        readings: &["ta"],
    },
    CharPinyinEntry {
        ch: '\u{9CCF}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{9CD0}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{9CD1}',
        readings: &["pang"],
    },
    CharPinyinEntry {
        ch: '\u{9CD2}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{9CD3}',
        readings: &["le"],
    },
    CharPinyinEntry {
        ch: '\u{9CD4}',
        readings: &["biao"],
    },
    CharPinyinEntry {
        ch: '\u{9CD5}',
        readings: &["xue"],
    },
    CharPinyinEntry {
        ch: '\u{9CD6}',
        readings: &["bie"],
    },
    CharPinyinEntry {
        ch: '\u{9CD7}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{9CD8}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{9CD9}',
        readings: &["yong"],
    },
    CharPinyinEntry {
        ch: '\u{9CDA}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{9CDB}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{9CDC}',
        readings: &["gui"],
    },
    CharPinyinEntry {
        ch: '\u{9CDD}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{9CDE}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{9CDF}',
        readings: &["zun"],
    },
    CharPinyinEntry {
        ch: '\u{9CE0}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{9CE1}',
        readings: &["gan"],
    },
    CharPinyinEntry {
        ch: '\u{9CE2}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{9CE3}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{9CE4}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{9E1F}',
        readings: &["niao"],
    },
    CharPinyinEntry {
        ch: '\u{9E20}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{9E21}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9E22}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{9E23}',
        readings: &["ming"],
    },
    CharPinyinEntry {
        ch: '\u{9E24}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{9E25}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{9E26}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{9E27}',
        readings: &["cang"],
    },
    CharPinyinEntry {
        ch: '\u{9E28}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{9E29}',
        readings: &["zhen"],
    },
    CharPinyinEntry {
        ch: '\u{9E2A}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{9E2B}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{9E2C}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{9E2D}',
        readings: &["ya"],
    },
    CharPinyinEntry {
        ch: '\u{9E2E}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{9E2F}',
        readings: &["yang"],
    },
    CharPinyinEntry {
        ch: '\u{9E30}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{9E31}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{9E32}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{9E33}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{9E35}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{9E36}',
        readings: &["si"],
    },
    CharPinyinEntry {
        ch: '\u{9E37}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{9E38}',
        readings: &["er"],
    },
    CharPinyinEntry {
        ch: '\u{9E39}',
        readings: &["gua"],
    },
    CharPinyinEntry {
        ch: '\u{9E3A}',
        readings: &["xiu"],
    },
    CharPinyinEntry {
        ch: '\u{9E3B}',
        readings: &["heng"],
    },
    CharPinyinEntry {
        ch: '\u{9E3C}',
        readings: &["zhou"],
    },
    CharPinyinEntry {
        ch: '\u{9E3D}',
        readings: &["ge"],
    },
    CharPinyinEntry {
        ch: '\u{9E3E}',
        readings: &["luan"],
    },
    CharPinyinEntry {
        ch: '\u{9E3F}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{9E40}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{9E41}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{9E42}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{9E43}',
        readings: &["juan"],
    },
    CharPinyinEntry {
        ch: '\u{9E44}',
        readings: &["gu", "hu"],
    },
    CharPinyinEntry {
        ch: '\u{9E45}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{9E46}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9E47}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{9E48}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{9E49}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{9E4A}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{9E4B}',
        readings: &["miao"],
    },
    CharPinyinEntry {
        ch: '\u{9E4C}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{9E4D}',
        readings: &["kun"],
    },
    CharPinyinEntry {
        ch: '\u{9E4E}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{9E4F}',
        readings: &["peng"],
    },
    CharPinyinEntry {
        ch: '\u{9E50}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{9E51}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{9E52}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{9E54}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{9E55}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{9E56}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{9E57}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{9E58}',
        readings: &["gu", "hu"],
    },
    CharPinyinEntry {
        ch: '\u{9E59}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{9E5A}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{9E5B}',
        readings: &["mei"],
    },
    CharPinyinEntry {
        ch: '\u{9E5C}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{9E5D}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9E5E}',
        readings: &["yao"],
    },
    CharPinyinEntry {
        ch: '\u{9E5F}',
        readings: &["weng"],
    },
    CharPinyinEntry {
        ch: '\u{9E60}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{9E61}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9E62}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9E63}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{9E64}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{9E66}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{9E67}',
        readings: &["zhe"],
    },
    CharPinyinEntry {
        ch: '\u{9E68}',
        readings: &["liu"],
    },
    CharPinyinEntry {
        ch: '\u{9E69}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{9E6A}',
        readings: &["jiao"],
    },
    CharPinyinEntry {
        ch: '\u{9E6B}',
        readings: &["jiu"],
    },
    CharPinyinEntry {
        ch: '\u{9E6C}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9E6D}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{9E6E}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{9E6F}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{9E70}',
        readings: &["ying"],
    },
    CharPinyinEntry {
        ch: '\u{9E71}',
        readings: &["hu"],
    },
    CharPinyinEntry {
        ch: '\u{9E72}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{9E73}',
        readings: &["guan"],
    },
    CharPinyinEntry {
        ch: '\u{9E74}',
        readings: &["shuang"],
    },
    CharPinyinEntry {
        ch: '\u{9E7E}',
        readings: &["cuo"],
    },
    CharPinyinEntry {
        ch: '\u{9E7F}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{9E80}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{9E82}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9E87}',
        readings: &["jun", "qun"],
    },
    CharPinyinEntry {
        ch: '\u{9E88}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{9E8B}',
        readings: &["mi"],
    },
    CharPinyinEntry {
        ch: '\u{9E91}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{9E92}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{9E93}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{9E96}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{9E9D}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{9E9F}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{9EA6}',
        readings: &["mai"],
    },
    CharPinyinEntry {
        ch: '\u{9EB8}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{9EB9}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{9EBB}',
        readings: &["ma"],
    },
    CharPinyinEntry {
        ch: '\u{9EBD}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{9EBE}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{9EC4}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{9EC7}',
        readings: &["tian"],
    },
    CharPinyinEntry {
        ch: '\u{9EC9}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{9ECD}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{9ECE}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{9ECF}',
        readings: &["nian"],
    },
    CharPinyinEntry {
        ch: '\u{9ED1}',
        readings: &["hei"],
    },
    CharPinyinEntry {
        ch: '\u{9ED4}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{9ED8}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{9EDB}',
        readings: &["dai"],
    },
    CharPinyinEntry {
        ch: '\u{9EDC}',
        readings: &["chu"],
    },
    CharPinyinEntry {
        ch: '\u{9EDD}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{9EDF}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{9EE0}',
        readings: &["xia"],
    },
    CharPinyinEntry {
        ch: '\u{9EE1}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{9EE2}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{9EE5}',
        readings: &["qing"],
    },
    CharPinyinEntry {
        ch: '\u{9EE7}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{9EE9}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{9EEA}',
        readings: &["can"],
    },
    CharPinyinEntry {
        ch: '\u{9EEF}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{9EF9}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{9EFB}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{9EFC}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{9EFE}',
        readings: &["min"],
    },
    CharPinyinEntry {
        ch: '\u{9F0B}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{9F0D}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{9F0E}',
        readings: &["ding"],
    },
    CharPinyinEntry {
        ch: '\u{9F10}',
        readings: &["nai"],
    },
    CharPinyinEntry {
        ch: '\u{9F12}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{9F13}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{9F17}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{9F19}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{9F20}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{9F22}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{9F29}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{9F2B}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{9F2C}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{9F2F}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{9F31}',
        readings: &["jing"],
    },
    CharPinyinEntry {
        ch: '\u{9F37}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{9F39}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{9F3B}',
        readings: &["bi"],
    },
    CharPinyinEntry {
        ch: '\u{9F3D}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{9F3E}',
        readings: &["han"],
    },
    CharPinyinEntry {
        ch: '\u{9F41}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{9F47}',
        readings: &["zha"],
    },
    CharPinyinEntry {
        ch: '\u{9F49}',
        readings: &["nang"],
    },
    CharPinyinEntry {
        ch: '\u{9F50}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{9F51}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{9F7F}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{9F80}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{9F81}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{9F82}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{9F83}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{9F84}',
        readings: &["ling"],
    },
    CharPinyinEntry {
        ch: '\u{9F85}',
        readings: &["bao"],
    },
    CharPinyinEntry {
        ch: '\u{9F86}',
        readings: &["tiao"],
    },
    CharPinyinEntry {
        ch: '\u{9F87}',
        readings: &["zi"],
    },
    CharPinyinEntry {
        ch: '\u{9F88}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{9F89}',
        readings: &["yu"],
    },
    CharPinyinEntry {
        ch: '\u{9F8A}',
        readings: &["chuo"],
    },
    CharPinyinEntry {
        ch: '\u{9F8B}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{9F8C}',
        readings: &["wo"],
    },
    CharPinyinEntry {
        ch: '\u{9F99}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{9F9A}',
        readings: &["gong"],
    },
    CharPinyinEntry {
        ch: '\u{9F9B}',
        readings: &["kan"],
    },
    CharPinyinEntry {
        ch: '\u{9F9F}',
        readings: &["gui", "jun", "qiu"],
    },
    CharPinyinEntry {
        ch: '\u{9FA0}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{9FA2}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{9FCD}',
        readings: &["gang"],
    },
    CharPinyinEntry {
        ch: '\u{9FCE}',
        readings: &["da", "ta"],
    },
    CharPinyinEntry {
        ch: '\u{9FCF}',
        readings: &["mai"],
    },
    CharPinyinEntry {
        ch: '\u{20164}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{20676}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{20CD0}',
        readings: &["bang"],
    },
    CharPinyinEntry {
        ch: '\u{2139A}',
        readings: &["pian"],
    },
    CharPinyinEntry {
        ch: '\u{21413}',
        readings: &["kang"],
    },
    CharPinyinEntry {
        ch: '\u{235CB}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{23C97}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{23C98}',
        readings: &["wu"],
    },
    CharPinyinEntry {
        ch: '\u{23E23}',
        readings: &["fen"],
    },
    CharPinyinEntry {
        ch: '\u{249DB}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{24A7D}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{24AC9}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{25532}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{25562}',
        readings: &["cao"],
    },
    CharPinyinEntry {
        ch: '\u{255A8}',
        readings: &["zao"],
    },
    CharPinyinEntry {
        ch: '\u{25ED7}',
        readings: &["cha"],
    },
    CharPinyinEntry {
        ch: '\u{26221}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{2648D}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{26676}',
        readings: &["gu"],
    },
    CharPinyinEntry {
        ch: '\u{2677C}',
        readings: &["lou", "lv"],
    },
    CharPinyinEntry {
        ch: '\u{26B5C}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{26C21}',
        readings: &["na", "nuo"],
    },
    CharPinyinEntry {
        ch: '\u{27FF9}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{28408}',
        readings: &["guang"],
    },
    CharPinyinEntry {
        ch: '\u{28678}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{28695}',
        readings: &["bian"],
    },
    CharPinyinEntry {
        ch: '\u{287E0}',
        readings: &["quan"],
    },
    CharPinyinEntry {
        ch: '\u{28B49}',
        readings: &["ban"],
    },
    CharPinyinEntry {
        ch: '\u{28C47}',
        readings: &["qiu"],
    },
    CharPinyinEntry {
        ch: '\u{28C4F}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{28C51}',
        readings: &["huang"],
    },
    CharPinyinEntry {
        ch: '\u{28C54}',
        readings: &["zun"],
    },
    CharPinyinEntry {
        ch: '\u{28E99}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{29F7E}',
        readings: &["an"],
    },
    CharPinyinEntry {
        ch: '\u{29F83}',
        readings: &["mian"],
    },
    CharPinyinEntry {
        ch: '\u{29F8C}',
        readings: &["kang"],
    },
    CharPinyinEntry {
        ch: '\u{2A7DD}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{2A8FB}',
        readings: &["lou"],
    },
    CharPinyinEntry {
        ch: '\u{2A917}',
        readings: &["liao"],
    },
    CharPinyinEntry {
        ch: '\u{2AA30}',
        readings: &["qu"],
    },
    CharPinyinEntry {
        ch: '\u{2AA36}',
        readings: &["she"],
    },
    CharPinyinEntry {
        ch: '\u{2AA58}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{2AFA2}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{2B127}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{2B128}',
        readings: &["chi"],
    },
    CharPinyinEntry {
        ch: '\u{2B137}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{2B138}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{2B1ED}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{2B300}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{2B363}',
        readings: &["tong"],
    },
    CharPinyinEntry {
        ch: '\u{2B36F}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{2B372}',
        readings: &["xiao"],
    },
    CharPinyinEntry {
        ch: '\u{2B37D}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{2B404}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{2B410}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{2B413}',
        readings: &["rou"],
    },
    CharPinyinEntry {
        ch: '\u{2B461}',
        readings: &["meng"],
    },
    CharPinyinEntry {
        ch: '\u{2B4E7}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{2B4EF}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{2B4F6}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{2B4F9}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{2B50D}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{2B50E}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{2B536}',
        readings: &["nie"],
    },
    CharPinyinEntry {
        ch: '\u{2B5AE}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{2B5AF}',
        readings: &["fu"],
    },
    CharPinyinEntry {
        ch: '\u{2B5B3}',
        readings: &["yun"],
    },
    CharPinyinEntry {
        ch: '\u{2B5E7}',
        readings: &["su"],
    },
    CharPinyinEntry {
        ch: '\u{2B5F4}',
        readings: &["zhan"],
    },
    CharPinyinEntry {
        ch: '\u{2B61C}',
        readings: &["wen"],
    },
    CharPinyinEntry {
        ch: '\u{2B61D}',
        readings: &["jue"],
    },
    CharPinyinEntry {
        ch: '\u{2B626}',
        readings: &["tao"],
    },
    CharPinyinEntry {
        ch: '\u{2B627}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{2B628}',
        readings: &["ti"],
    },
    CharPinyinEntry {
        ch: '\u{2B62A}',
        readings: &["yuan"],
    },
    CharPinyinEntry {
        ch: '\u{2B62C}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{2B695}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{2B696}',
        readings: &["ci"],
    },
    CharPinyinEntry {
        ch: '\u{2B6AD}',
        readings: &["lie"],
    },
    CharPinyinEntry {
        ch: '\u{2B6ED}',
        readings: &["kuang"],
    },
    CharPinyinEntry {
        ch: '\u{2B7A9}',
        readings: &["men"],
    },
    CharPinyinEntry {
        ch: '\u{2B7C5}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{2B7E6}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{2B7F9}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{2B7FC}',
        readings: &["da"],
    },
    CharPinyinEntry {
        ch: '\u{2B806}',
        readings: &["kui"],
    },
    CharPinyinEntry {
        ch: '\u{2B80A}',
        readings: &["xuan"],
    },
    CharPinyinEntry {
        ch: '\u{2B81C}',
        readings: &["ni"],
    },
    CharPinyinEntry {
        ch: '\u{2B8B8}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{2BAC7}',
        readings: &["e"],
    },
    CharPinyinEntry {
        ch: '\u{2BB5F}',
        readings: &["ou", "qu"],
    },
    CharPinyinEntry {
        ch: '\u{2BB62}',
        readings: &["lun"],
    },
    CharPinyinEntry {
        ch: '\u{2BB7C}',
        readings: &["lao"],
    },
    CharPinyinEntry {
        ch: '\u{2BB83}',
        readings: &["shan"],
    },
    CharPinyinEntry {
        ch: '\u{2BC1B}',
        readings: &["xing"],
    },
    CharPinyinEntry {
        ch: '\u{2BD77}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{2BD87}',
        readings: &["die"],
    },
    CharPinyinEntry {
        ch: '\u{2BDF7}',
        readings: &["xin"],
    },
    CharPinyinEntry {
        ch: '\u{2BE29}',
        readings: &["kou"],
    },
    CharPinyinEntry {
        ch: '\u{2C029}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{2C02A}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{2C0A9}',
        readings: &["jia"],
    },
    CharPinyinEntry {
        ch: '\u{2C0CA}',
        readings: &["zhi"],
    },
    CharPinyinEntry {
        ch: '\u{2C1D5}',
        readings: &["wan"],
    },
    CharPinyinEntry {
        ch: '\u{2C1D9}',
        readings: &["bei"],
    },
    CharPinyinEntry {
        ch: '\u{2C1F9}',
        readings: &["guo"],
    },
    CharPinyinEntry {
        ch: '\u{2C27C}',
        readings: &["ou"],
    },
    CharPinyinEntry {
        ch: '\u{2C288}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{2C2A4}',
        readings: &["chan"],
    },
    CharPinyinEntry {
        ch: '\u{2C317}',
        readings: &["he"],
    },
    CharPinyinEntry {
        ch: '\u{2C35B}',
        readings: &["li"],
    },
    CharPinyinEntry {
        ch: '\u{2C361}',
        readings: &["dang"],
    },
    CharPinyinEntry {
        ch: '\u{2C364}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{2C488}',
        readings: &["que"],
    },
    CharPinyinEntry {
        ch: '\u{2C494}',
        readings: &["geng"],
    },
    CharPinyinEntry {
        ch: '\u{2C497}',
        readings: &["lan"],
    },
    CharPinyinEntry {
        ch: '\u{2C542}',
        readings: &["long"],
    },
    CharPinyinEntry {
        ch: '\u{2C613}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{2C618}',
        readings: &["dan"],
    },
    CharPinyinEntry {
        ch: '\u{2C621}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{2C629}',
        readings: &["ting"],
    },
    CharPinyinEntry {
        ch: '\u{2C62B}',
        readings: &["huan"],
    },
    CharPinyinEntry {
        ch: '\u{2C62C}',
        readings: &["qian"],
    },
    CharPinyinEntry {
        ch: '\u{2C62D}',
        readings: &["chen"],
    },
    CharPinyinEntry {
        ch: '\u{2C62F}',
        readings: &["zhun"],
    },
    CharPinyinEntry {
        ch: '\u{2C642}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{2C64A}',
        readings: &["mo"],
    },
    CharPinyinEntry {
        ch: '\u{2C64B}',
        readings: &["xiang"],
    },
    CharPinyinEntry {
        ch: '\u{2C72C}',
        readings: &["man"],
    },
    CharPinyinEntry {
        ch: '\u{2C72F}',
        readings: &["liang"],
    },
    CharPinyinEntry {
        ch: '\u{2C79F}',
        readings: &["pin"],
    },
    CharPinyinEntry {
        ch: '\u{2C7C1}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{2C7FD}',
        readings: &["dong"],
    },
    CharPinyinEntry {
        ch: '\u{2C8D9}',
        readings: &["xu"],
    },
    CharPinyinEntry {
        ch: '\u{2C8DE}',
        readings: &["zhu"],
    },
    CharPinyinEntry {
        ch: '\u{2C8E1}',
        readings: &["jian"],
    },
    CharPinyinEntry {
        ch: '\u{2C8F3}',
        readings: &["hen"],
    },
    CharPinyinEntry {
        ch: '\u{2C907}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{2C90A}',
        readings: &["shi"],
    },
    CharPinyinEntry {
        ch: '\u{2C91D}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{2CA02}',
        readings: &["qi"],
    },
    CharPinyinEntry {
        ch: '\u{2CA0E}',
        readings: &["you"],
    },
    CharPinyinEntry {
        ch: '\u{2CA7D}',
        readings: &["xun"],
    },
    CharPinyinEntry {
        ch: '\u{2CAA9}',
        readings: &["nong"],
    },
    CharPinyinEntry {
        ch: '\u{2CB29}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{2CB2D}',
        readings: &["lun"],
    },
    CharPinyinEntry {
        ch: '\u{2CB2E}',
        readings: &["chang"],
    },
    CharPinyinEntry {
        ch: '\u{2CB31}',
        readings: &["jin"],
    },
    CharPinyinEntry {
        ch: '\u{2CB38}',
        readings: &["shu"],
    },
    CharPinyinEntry {
        ch: '\u{2CB39}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{2CB3B}',
        readings: &["lu"],
    },
    CharPinyinEntry {
        ch: '\u{2CB3F}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{2CB41}',
        readings: &["mu"],
    },
    CharPinyinEntry {
        ch: '\u{2CB4A}',
        readings: &["du"],
    },
    CharPinyinEntry {
        ch: '\u{2CB4E}',
        readings: &["hong"],
    },
    CharPinyinEntry {
        ch: '\u{2CB5A}',
        readings: &["chun"],
    },
    CharPinyinEntry {
        ch: '\u{2CB5B}',
        readings: &["bo"],
    },
    CharPinyinEntry {
        ch: '\u{2CB64}',
        readings: &["hou"],
    },
    CharPinyinEntry {
        ch: '\u{2CB69}',
        readings: &["weng"],
    },
    CharPinyinEntry {
        ch: '\u{2CB6C}',
        readings: &["hui"],
    },
    CharPinyinEntry {
        ch: '\u{2CB6F}',
        readings: &["pie"],
    },
    CharPinyinEntry {
        ch: '\u{2CB73}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{2CB76}',
        readings: &["hei"],
    },
    CharPinyinEntry {
        ch: '\u{2CB78}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{2CB7C}',
        readings: &["sui"],
    },
    CharPinyinEntry {
        ch: '\u{2CBB1}',
        readings: &["yin"],
    },
    CharPinyinEntry {
        ch: '\u{2CBBF}',
        readings: &["gai"],
    },
    CharPinyinEntry {
        ch: '\u{2CBC0}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{2CBCE}',
        readings: &["tui"],
    },
    CharPinyinEntry {
        ch: '\u{2CC56}',
        readings: &["di"],
    },
    CharPinyinEntry {
        ch: '\u{2CC5F}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{2CCF5}',
        readings: &["pi"],
    },
    CharPinyinEntry {
        ch: '\u{2CCF6}',
        readings: &["jiong"],
    },
    CharPinyinEntry {
        ch: '\u{2CCFD}',
        readings: &["shen"],
    },
    CharPinyinEntry {
        ch: '\u{2CCFF}',
        readings: &["tu"],
    },
    CharPinyinEntry {
        ch: '\u{2CD02}',
        readings: &["fei"],
    },
    CharPinyinEntry {
        ch: '\u{2CD03}',
        readings: &["huo"],
    },
    CharPinyinEntry {
        ch: '\u{2CD0A}',
        readings: &["lin"],
    },
    CharPinyinEntry {
        ch: '\u{2CD8B}',
        readings: &["ju"],
    },
    CharPinyinEntry {
        ch: '\u{2CD8D}',
        readings: &["tuo"],
    },
    CharPinyinEntry {
        ch: '\u{2CD8F}',
        readings: &["wei"],
    },
    CharPinyinEntry {
        ch: '\u{2CD90}',
        readings: &["zhao"],
    },
    CharPinyinEntry {
        ch: '\u{2CD9F}',
        readings: &["la"],
    },
    CharPinyinEntry {
        ch: '\u{2CDA0}',
        readings: &["lian"],
    },
    CharPinyinEntry {
        ch: '\u{2CDA8}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{2CDAD}',
        readings: &["ji"],
    },
    CharPinyinEntry {
        ch: '\u{2CDAE}',
        readings: &["xi"],
    },
    CharPinyinEntry {
        ch: '\u{2CDD5}',
        readings: &["bu"],
    },
    CharPinyinEntry {
        ch: '\u{2CE18}',
        readings: &["yan"],
    },
    CharPinyinEntry {
        ch: '\u{2CE1A}',
        readings: &["yue"],
    },
    CharPinyinEntry {
        ch: '\u{2CE23}',
        readings: &["xian"],
    },
    CharPinyinEntry {
        ch: '\u{2CE26}',
        readings: &["zhuo"],
    },
    CharPinyinEntry {
        ch: '\u{2CE2A}',
        readings: &["fan"],
    },
    CharPinyinEntry {
        ch: '\u{2CE7C}',
        readings: &["xie"],
    },
    CharPinyinEntry {
        ch: '\u{2CE88}',
        readings: &["yi"],
    },
    CharPinyinEntry {
        ch: '\u{2CE93}',
        readings: &["chu"],
    },
];

/// 查询单字全部读音；未收录返回 `None`（kTGHZ 外生僻字）。
#[must_use]
pub fn char_pinyin(ch: char) -> Option<&'static [&'static str]> {
    CHAR_PINYIN
        .binary_search_by_key(&ch, |entry| entry.ch)
        .ok()
        .map(|index| CHAR_PINYIN[index].readings)
}

#[cfg(test)]
mod tests {
    use super::{char_pinyin, CHAR_PINYIN};

    #[test]
    fn 表按字升序() {
        for pair in CHAR_PINYIN.windows(2) {
            assert!(pair[0].ch < pair[1].ch, "表必须按 char 升序");
        }
    }

    #[test]
    fn 多音字全形态() {
        // D-22：全部读音形态建键（集合语义断言，不依赖 kTGHZ 行的读音顺序）。
        for (ch, readings) in [
            ('谁', &["shei", "shui"]),
            ('曾', &["ceng", "zeng"]),
            ('了', &["le", "liao"]),
            ('乐', &["le", "yue"]),
        ] {
            let got = char_pinyin(ch).expect("常见多音字应被收录");
            for reading in readings {
                assert!(
                    got.contains(reading),
                    "字 {ch} 应含读音 {reading}，实际 {got:?}"
                );
            }
        }
    }

    #[test]
    fn ü归一为v() {
        assert!(char_pinyin('绿').unwrap().contains(&"lv"), "绿 应含 lv");
        assert_eq!(char_pinyin('女'), Some(&["nv"][..]));
    }

    #[test]
    fn 调符已剥离() {
        assert_eq!(char_pinyin('张'), Some(&["zhang"][..]));
        assert_eq!(char_pinyin('三'), Some(&["san"][..]));
    }

    #[test]
    fn 未收录字返回none() {
        assert_eq!(char_pinyin('𠀀'), None);
    }

    #[test]
    fn 常见字抽查() {
        for (ch, expect) in [
            ('你', &["ni"][..]),
            ('好', &["hao"][..]),
            ('李', &["li"][..]),
            ('王', &["wang"][..]),
            ('国', &["guo"][..]),
        ] {
            assert_eq!(char_pinyin(ch), Some(expect), "字 {ch} 读音不符");
        }
    }
}
