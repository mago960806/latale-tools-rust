# LaTale 客户端资源加载器 - 关卡系统文档

## 概述

本文档描述 LaTale 游戏中的关卡/地图场景资源格式 (.STG)。

## 版本信息

| 版本 | 日期 | 变更内容 |
|-----|------|---------|
| 0.4 | 2004/12/30 | 最新版本 |

## 目录

- [系统架构](#系统架构)
- [STG 文件格式](#stg-文件格式)
- [数据结构定义](#数据结构定义)
- [二进制格式布局](#二进制格式布局)
- [ASCII 格式说明](#ascii-格式说明)

---

## 系统架构

### 关卡层级结构

```
Stage (关卡)
  └─ MapGroup (地图组)
       └─ MapInfo (地图信息)
            ├─ BGIndex (背景索引)
            ├─ FormFile (地形形状文件)
            ├─ AttributeFile (地形属性文件)
            └─ MiniMapFile (小地图文件)
```

### 三级数据结构

关卡系统采用三级层级结构组织数据：

1. **Stage (关卡)** - 顶层容器，包含整个游戏关卡的信息
2. **MapGroup (地图组)** - 中间层，包含一个关卡内的多个相关地图
3. **MapInfo (地图信息)** - 底层，包含单个地图的具体资源文件引用

---

## STG 文件格式

### MAP_LINK 枚举值

地图链接类型用于定义地图之间的连接方式：

| 值 | 名称 | 描述 |
|----|------|------|
| 0 | NULL | 无链接（独立地图） |
| 1 | HORIZONTAL | 水平链接（左右连接） |
| 2 | VERTICAL | 垂直链接（上下连接） |

### GROUP_TYPE 位掩码值

地图组类型使用位掩码定义，可以组合使用：

| 位掩码值 | 名称 | 描述 |
|---------|------|------|
| 0x01 | FIELD | 战场地图 |
| 0x02 | MINIMAP | 小地图 |
| 0x04 | EVENT | 事件地图 |
| 0x08 | TOWN | 城镇地图 |
| 0x10 | DUNGEON | 副本地图 |
| 0x20 | PVP | PVP 地图 |
| 0x40 | CINEMATIC | 过场动画地图 |
| 0x80 | SPECIAL | 特殊地图 |

### 物理参数存储方式

物理参数在文件中以整数形式存储，实际使用时需要转换为浮点数：

**转换公式：**
```
浮点值 = 整数值 × 0.001
```

**示例：**
- 文件中存储：`1000`
- 实际值：`1.0`
- 文件中存储：`1500`
- 实际值：`1.5`

**原因：** 使用定点数格式可以在不损失精度的情况下减少浮点运算的复杂性，同时保持跨平台的一致性。

---

## 数据结构定义

### Stage (关卡) 结构

关卡是整个系统的顶层容器，包含一个完整游戏区域的所有信息。

**字段说明：**

| 字段 | 类型 | 描述 |
|------|------|------|
| StageID | int | 关卡唯一标识符 |
| StageName | string | 关卡名称 |
| SyncRegionWidth | int | 同步区域宽度（像素） |
| SyncRegionHeight | int | 同步区域高度（像素） |
| PaletteFile | string | 地形调色板文件路径 |
| GroupCount | int | 包含的地图组数量 |

### MapGroup (地图组) 结构

地图组是关卡的子容器，包含一个区域内的多个地图。

**字段说明：**

| 字段 | 类型 | 描述 |
|------|------|------|
| GroupID | int | 地图组唯一标识符 |
| GroupName | string | 地图组名称 |
| MapLink | int | 地图链接类型（见 MAP_LINK 枚举） |
| BGID | int | 背景资源 ID |
| Type | int | 地图组类型（见 GROUP_TYPE 位掩码） |
| BGFile | string | 背景图像文件路径 |
| BGMFile | string | 背景音乐文件路径 |
| SoundEffectType | int | 音效类型 |
| ThemeIcon | int | 小地图图标资源 ID |
| ThemeIconIndex | int | 小地图图标索引 |
| MapCount | int | 包含的地图数量 |

**物理参数字段：**

| 字段 | 类型 | 存储格式 | 实际值示例 |
|------|------|----------|-----------|
| Drop_Speed | int | 整数×0.001 | 1000 → 1.0 |
| Max_Drop_Speed | int | 整数×0.001 | 1500 → 1.5 |
| Speed_X | int | 整数×0.001 | 500 → 0.5 |
| Speed_Y | int | 整数×0.001 | 1000 → 1.0 |
| Rope_Speed_Y | int | 整数×0.001 | 300 → 0.3 |
| Rope_Speed_X | int | 整数×0.001 | 200 → 0.2 |

### MapInfo (地图信息) 结构

地图信息是系统的最小数据单元，引用具体的资源文件。

**字段说明：**

| 字段 | 类型 | 描述 |
|------|------|------|
| BGIndex | int | 背景索引 |
| MapName | string | 地图名称 |
| FormFile | string | 地形形状文件路径 (.FORM) |
| AttributeFile | string | 地形属性文件路径 (.ATTR) |
| MiniMapFile | string | 小地图图像文件路径 (.TGA) |

---

## 二进制格式布局

### 文件结构概览

```
文件头:
  Offset  Type          Size    Field          描述
  0x00    int           4       StageCount     关卡数量

  对于每个关卡:
    0x00    int           4       StageID        关卡 ID
    0x04    int           4       SyncRegionWidth 同步区域宽度
    0x08    int           4       SyncRegionHeight 同步区域高度
    0x0C    char[64]      64      StageName      关卡名称
    0x4C    char[64]      64      PaletteFile    调色板文件
    0x8C    int           4       GroupCount     地图组数量

    对于每个地图组:
      0x00    int           4       GroupID        组 ID
      0x04    int           4       MapLink        地图链接
      0x08    int           4       BGID           背景 ID
      0x0C    int           4       Type           类型
      0x10    char[64]      64      GroupName      组名称
      0x50    char[64]      64      BGFile         背景文件
      0x90    char[64]      64      BGMFile        背景音乐文件

      // 物理参数
      0xD0    int           4       Gravity        重力 (×0.001)
      0xD4    int           4       MaxDrop        最大下落 (×0.001)
      0xD8    int           4       VelocityX      X 速度 (×0.001)
      0xDC    int           4       JumpSpeed      跳跃速度 (×0.001)
      0xE0    int           4       UpdownVelocity 上下速度 (×0.001)
      0xE4    int           4       HangingVelocity 悬挂速度 (×0.001)

      // 扩展数据 (v0.4+)
      0xE8    int           4       SoundEffectType 音效类型
      0xEC    int           4       MiniMapIconID  小地图图标 ID
      0xF0    int           4       MiniMapResID   小地图资源 ID

      0xF4    int           4       MapCount       地图数量

      对于每个地图:
        0x00    int           4       BGIndex        背景索引
        0x04    char[64]      64      MapName        地图名称
        0x44    char[64]      64      FormFile       地形形状文件
        0x84    char[64]      64      AttributeFile  地形属性文件
        0xC4    char[64]      64      MiniMapFile    小地图文件
```

### 偏移量计算说明

所有偏移量都是相对于其所属结构体的起始位置：

- **关卡级别**：相对于关卡数据块的起始位置
- **地图组级别**：相对于地图组数据块的起始位置
- **地图级别**：相对于地图数据块的起始位置

### 大小端序

所有数值类型使用 **小端序 (Little-Endian)** 字节序存储。

---

## ASCII 格式说明

### 格式结构

ASCII 格式使用类 C 的语法，通过花括号定义层级结构。

### 完整示例

```
STAGE_HEADER
{
    StageCount = 1

    STAGE
    {
        StageID = 1
        StageName = "Forest of Beginning"
        SyncRegionWidth = 2000
        SyncRegionHeight = 1000
        PaletteFile = "TERRAINPALLET.PAL"

        GROUP
        {
            GroupID = 1
            GroupName = "Main Area"
            MapLink = 0
            BGID = 1
            Type = 0
            BGFile = "BG_001.BGI"
            BGMFile = "BGM_001.OGG"
            SoundEffect = 0
            ThemeIcon = 100
            ThemeIconIndex = 0
            Drop_Speed = 1000
            Max_Drop_Speed = 1500
            Speed_X = 500
            Speed_Y = 1000
            Rope_Speed_Y = 300
            Rope_Speed_X = 200

            MAP
            {
                MapName = "Main Map"
                BGIndex = 1
                FormFile = "FORM_001.FORM"
                AttributeFile = "ATTR_001.ATTR"
                MiniMapFile = "MINIMAP_001.TGA"
            }
        }
    }
}
```

### 字段定义规范

#### 关卡级字段

| 字段 | 类型 | 描述 |
|------|------|------|
| StageID | 整数 | 关卡 ID |
| StageName | 字符串 | 关卡名称（带引号） |
| SyncRegionWidth | 整数 | 同步区域宽度（像素） |
| SyncRegionHeight | 整数 | 同步区域高度（像素） |
| PaletteFile | 字符串 | 调色板文件名（带引号） |

#### 地图组级字段

| 字段 | 类型 | 描述 |
|------|------|------|
| GroupID | 整数 | 组 ID |
| GroupName | 字符串 | 组名称（带引号） |
| MapLink | 整数 | 地图链接类型（0=无, 1=水平, 2=垂直） |
| BGID | 整数 | 背景 ID |
| Type | 整数 | 地图组类型（位掩码） |
| BGFile | 字符串 | 背景文件名（带引号） |
| BGMFile | 字符串 | 背景音乐文件名（带引号） |
| SoundEffect | 整数 | 音效类型 |
| ThemeIcon | 整数 | 小地图图标资源 ID |
| ThemeIconIndex | 整数 | 小地图图标索引 |

#### 物理参数字段

| 字段 | 类型 | 描述 |
|------|------|------|
| Drop_Speed | 整数 | 重力（实际值 ×1000） |
| Max_Drop_Speed | 整数 | 最大下落速度（实际值 ×1000） |
| Speed_X | 整数 | X 方向速度（实际值 ×1000） |
| Speed_Y | 整数 | Y 方向速度（实际值 ×1000） |
| Rope_Speed_Y | 整数 | 绳索上下速度（实际值 ×1000） |
| Rope_Speed_X | 整数 | 绳索左右速度（实际值 ×1000） |

#### 地图级字段

| 字段 | 类型 | 描述 |
|------|------|------|
| MapName | 字符串 | 地图名称（带引号） |
| BGIndex | 整数 | 背景索引 |
| FormFile | 字符串 | 地形形状文件名（带引号） |
| AttributeFile | 字符串 | 地形属性文件名（带引号） |
| MiniMapFile | 字符串 | 小地图文件名（带引号） |

### 格式规则

1. **字符串值**：必须用双引号包裹
2. **数值**：可以是十进制整数
3. **注释**：不支持注释（原始格式）
4. **空白**：空格、制表符、换行符被忽略
5. **层级**：使用花括号 `{}` 定义结构层级
6. **赋值**：使用等号 `=` 赋值
7. **终止**：每个赋值语句以分号 `;` 结尾（可选，某些解析器要求）

---

## 参考

- [通用参考文档](00_common_reference.md)
- [地形系统文档](03_terrain_system.md)
