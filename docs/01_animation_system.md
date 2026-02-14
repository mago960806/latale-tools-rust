# LaTale 客户端资源文件 - 动画系统格式规范

## 概述

本文档描述 LaTale 游戏中角色动画 (.SEQ) 和怪物/NPC 动画 (.MOB) 资源文件的格式规范。

## 版本信息

| 版本 | 日期 | 变更内容 |
|-----|------|---------|
| 2.0 | 2004/10/12 | 最新版本 |

## 目录

- [文件类型](#文件类型)
- [部件层枚举](#部件层枚举)
- [动画索引枚举](#动画索引枚举)
- [角色动画格式](#角色动画格式-seq)
- [怪物动画格式](#怪物动画格式-mob)
- [二进制格式布局](#二进制格式布局)
- [ASCII 格式规范](#ascii-格式规范)

---

## 文件类型

| 扩展名 | 类型 | 用途 | 当前版本 |
|-------|------|------|---------|
| .SEQ | 角色动画 | 角色动画数据 | 2.0 |
| .MOB | 怪物动画 | 怪物/NPC 动画数据 | 2.0 |

## 文件路径

```
DATA/
├── ANITABLE/              # 动画表文件
│   ├── ANI_STAND.SEQ       # 站立动画
│   ├── ANI_WALK.SEQ        # 行走动画
│   ├── ANI_JUMP.SEQ        # 跳跃动画
│   └── ...
└── CHAR/
    ├── CHARLAYER/         # 角色图层
    └── MONSTER/           # 怪物/NPC 资源
```

---

## 部件层枚举 (PART_LAYER)

角色动画由 35 个部件层组成，每个部件可以独立设置位置、旋转、缩放等属性。

### 枚举值定义

| 枚举名 | 值 | 类别 | 描述 |
|--------|---|------|------|
| PL_NULL | -1 | - | 空层 |
| PL_BODY | 0 | 基础层 | 身体 |
| PL_HEAD | 1 | 基础层 | 头部 |
| PL_ARM_OUT | 2 | 外层(右) | 右臂 |
| PL_ARM_IN | 3 | 内层(左) | 左臂 |
| PL_LEG_OUT | 4 | 外层(右) | 右腿 |
| PL_LEG_IN | 5 | 内层(左) | 左腿 |
| PL_HAND_OUT | 6 | 外层(右) | 右手 |
| PL_HAND_IN | 7 | 内层(左) | 左手 |
| PL_FOOT_OUT | 8 | 外层(右) | 右脚 |
| PL_FOOT_IN | 9 | 内层(左) | 左脚 |
| PL_BLOUSE | 10 | 装备层 | 衬衫 |
| PL_PANTS | 11 | 装备层 | 裤子 |
| PL_COAT | 12 | 装备层 | 外套 |
| PL_BLOUSE_ARM_OUT | 13 | 装备-外层 | 衬衫右臂 |
| PL_PANTS_LEG_OUT | 14 | 装备-外层 | 裤子右腿 |
| PL_FOOT_LEG_OUT | 15 | 装备-外层 | 裤子右脚 |
| PL_GLOVE_OUT | 16 | 装备-外层 | 右手套 |
| PL_SHOE_OUT | 17 | 装备-外层 | 右鞋 |
| PL_BLOUSE_ARM_IN | 18 | 装备-内层 | 衬衫左臂 |
| PL_PANTS_LEG_IN | 19 | 装备-内层 | 裤子左腿 |
| PL_FOOT_LEG_IN | 20 | 装备-内层 | 裤子左脚 |
| PL_GLOVE_IN | 21 | 装备-内层 | 左手套 |
| PL_SHOE_IN | 22 | 装备-内层 | 左鞋 |
| PL_CAP_FRONT | 23 | 头部部件 | 帽子前部 |
| PL_GOGGLE | 24 | 头部部件 | 护目镜 |
| PL_EAR | 25 | 头部部件 | 耳朵 |
| PL_HAIR_FRONT | 26 | 头部部件 | 前发 |
| PL_HAIR_REAR | 27 | 头部部件 | 后发 |
| PL_FACE | 28 | 头部部件 | 脸部 |
| PL_MAKEUP | 29 | 头部部件 | 化妆 |
| PL_CAP_REAR | 30 | 头部部件 | 帽子后部 |
| PL_LAST | 31 | - | 最后一个层 |

### 层级关系

```
层级顺序（从前到后）：
1. 外层右部件 (ARM_OUT, LEG_OUT, HAND_OUT, FOOT_OUT 等)
2. 装备外层部件 (BLOUSE_ARM_OUT, PANTS_LEG_OUT 等)
3. 基础层 (BODY, HEAD)
4. 装备层 (BLOUSE, PANTS, COAT)
5. 内层左部件 (ARM_IN, LEG_IN, HAND_IN, FOOT_IN 等)
6. 装备内层部件 (BLOUSE_ARM_IN, PANTS_LEG_IN 等)
```

---

## 动画索引枚举 (ANIMATION_INDEX)

### 角色动画索引 (0-99)

| 索引 | 名称 | 描述 |
|-----|------|------|
| 0 | ANI_CHARACTER_STAND | 站立 |
| 1 | ANI_CHARACTER_WALK | 行走 |
| 2 | ANI_CHARACTER_JUMP | 跳跃 |
| 3 | ANI_CHARACTER_FLY | 飞行 |
| 4 | ANI_CHARACTER_ATTACK00 | 攻击 0 |
| 5 | ANI_CHARACTER_ATTACK01 | 攻击 1 |
| 6 | ANI_CHARACTER_CLIMB_LADDER_UP | 梯子向上 |
| 7 | ANI_CHARACTER_CLIMB_LADDER_WAIT | 梯子等待 |
| 8 | ANI_CHARACTER_CLIMB_LADDER_DOWN | 梯子向下 |
| 9 | ANI_CHARACTER_CLIMB_ROPE_UP | 绳子向上 |
| 10 | ANI_CHARACTER_CLIMB_ROPE_WAIT | 绳子等待 |
| 11 | ANI_CHARACTER_CLIMB_ROPE_DOWN | 绳子向下 |
| 12 | ANI_CHARACTER_HANGING_MOVE | 悬挂移动 |
| 13 | ANI_CHARACTER_HANGING_WAIT | 悬挂等待 |

### 怪物动画索引 (100-199)

| 索引 | 名称 | 描述 |
|-----|------|------|
| 100 | ANI_MONSTER_STAND | 怪物站立 |
| 101 | ANI_MONSTER_WALK | 怪物行走 |
| 102 | ANI_MONSTER_ATTACK | 怪物攻击 |

### NPC 动画索引 (200-299)

| 索引 | 范围 | 描述 |
|-----|------|------|
| 200-299 | ANI_NPC_BEGIN 到 ANI_NPC_END | NPC 动画 |

---

## 角色动画格式 (.SEQ)

### 文件头结构

```
偏移量  类型          字段名            描述
0x00    unsigned int  AnimationIndex    动画索引 (ANIMATION_INDEX)
0x04    float         AccumulateTime    累积时间 (毫秒)
0x08    int           SequenceCount     序列数量
```

### 动画序列结构

对于每个序列：

```
偏移量  类型          字段名            描述
0x00    int           SequenceNumber    序列号
0x04    float         DelayTime         延迟时间 (毫秒)
0x08    int           PartCount         部件数量
```

### 部件数据结构

对于每个部件：

```
偏移量  类型          字段名            描述
0x00    PART_LAYER    PartLayer         部件层类型
0x04    int           RelativeX         相对 X 坐标
0x08    int           RelativeY         相对 Y 坐标
0x0C    int           RotationDegree    旋转角度 (度)
0x10    int           ResourceIndex     资源索引 (0 = 使用父级资源)
0x14    bool          Visible           可见性
0x15    bool          Flip              翻转标志
0x16    -             -                 填充到 4 字节边界
```

---

## 怪物动画格式 (.MOB)

### 文件头结构

```
偏移量  类型            字段名            描述
0x00    unsigned int    ClassID           类 ID
0x04    int             AnimationCount     动画数量
```

### 怪物大小枚举 (MONSTER_INDEX)

| 值 | 名称 | 碰撞高度 (像素) |
|---|------|-----------------|
| 0 | MON_TINY | 2 × RECT_SIZE (70) |
| 1 | MON_SMALL | 1 × RECT_SIZE (35) |
| 2 | MON_MED | 2 × RECT_SIZE (70) |
| 3 | MON_BIG | 3 × RECT_SIZE (105) |

其中 RECT_SIZE = 35

### 怪物类型枚举 (MONSTER_TYPE)

| 值 | 名称 | 描述 |
|---|------|------|
| 0 | MT_NORMAL | 普通怪物 |
| 1 | MT_BOSS | Boss 怪物 |
| 2 | MT_NPC | NPC |

### 动画数据结构

对于每个动画：

```
偏移量  类型              字段名            描述
0x00    ANIMATION_INDEX    AnimationIndex    动画索引
0x04    MONSTER_INDEX     MonsterSize       怪物大小
0x08    MONSTER_TYPE      MonsterType       怪物类型
0x0C    float             AccumulateTime    累积时间
0x10    int               FrameCountX       横向帧数
0x14    int               FrameCountY       纵向帧数
0x18    int               FrameWidth        帧宽度
0x1C    int               FrameHeight       帧高度
0x20    char[64]          ImageName         图片文件名
0x60    int               FrameCount        帧数量
```

### 帧数据结构

对于每个帧：

```
偏移量  类型          字段名            描述
0x00    int           FrameNumber       帧序号
0x04    int           ZOrder            Z 顺序 (绘制顺序)
0x08    bool          Visible           可见性
0x09    -             -                 填充
0x0C    int           ResourceIndex     资源索引
0x10    int           PosX              X 坐标
0x14    int           PosY              Y 坐标
0x18    int           Rotation          旋转角度
0x1C    float         ScaleX            X 缩放
0x20    float         ScaleY            Y 缩放
0x24    float         ColorR            颜色 R (0.0-1.0)
0x28    float         ColorG            颜色 G (0.0-1.0)
0x2C    float         ColorB            颜色 B (0.0-1.0)
0x30    float         ColorA            颜色 A (0.0-1.0)
0x34    float         Delay             延迟时间
```

---

## 二进制格式布局

### 角色 (.SEQ) 完整布局

```
文件头:
  Offset  Type          Field
  0x00    unsigned int  AnimationIndex
  0x04    float         AccumulateTime
  0x08    int           SequenceCount

  对于每个序列:
    0x00    int           SequenceNumber
    0x04    float         DelayTime
    0x08    int           PartCount

    对于每个部件:
      0x00    PART_LAYER    PartLayer
      0x04    int           RelativeX
      0x08    int           RelativeY
      0x0C    int           RotationDegree
      0x10    int           ResourceIndex
      0x14    bool          Visible
      0x15    bool          Flip
      0x16    [padding]     2 bytes
```

### 怪物 (.MOB) 完整布局

```
文件头:
  Offset  Type            Field
  0x00    unsigned int    ClassID
  0x04    int             AnimationCount

  对于每个动画:
    0x00    ANIMATION_INDEX  AnimationIndex
    0x04    MONSTER_INDEX   MonsterSize
    0x08    MONSTER_TYPE    MonsterType
    0x0C    float           AccumulateTime
    0x10    int             FrameCountX
    0x14    int             FrameCountY
    0x18    int             FrameWidth
    0x1C    int             FrameHeight
    0x20    char[64]        ImageName
    0x60    int             FrameCount

    对于每个帧:
      0x00    int             FrameNumber
      0x04    int             ZOrder
      0x08    bool            Visible
      0x09    [padding]       3 bytes
      0x0C    int             ResourceIndex
      0x10    int             PosX
      0x14    int             PosY
      0x18    int             Rotation
      0x1C    float           ScaleX
      0x20    float           ScaleY
      0x24    float           ColorR
      0x28    float           ColorG
      0x2C    float           ColorB
      0x30    float           ColorA
      0x34    float           Delay
```

---

## ASCII 格式规范

### 角色 (.SEQ) ASCII 格式

```
ANITABLE_HEADER
{
    ANI_CHARACTER_STAND
    {
        ACCUMULATE_TIME = 1000

        SEQ = 0 100
        {
            PL_BODY = 0 0 0 1 0
            PL_HEAD = 0 0 0 1 0
            PL_ARM_OUT = 30 0 0 1 0
            PL_HAND_OUT = 50 0 0 1 0
            ...
        }

        SEQ = 1 100
        {
            PL_BODY = 0 0 0 1 0
            PL_HEAD = 0 0 0 1 0
            ...
        }
    }
}
```

**字段说明：**
- `SEQ = 序号 延迟时间` - 定义动画序列
- `PL_层名 = 相对X 相对Y 旋转度 资源索引 可见 翻转` - 定义部件层

### 怪物 (.MOB) ASCII 格式

```
ANITABLE_HEADER
{
    ANI_MONSTER_STAND
    {
        SIZE = TINY 2 4 50 50
        {
            LAYERNO = 0 100 8
            NAME = "MonsterName"
            IMAGE = "monster.tga"
            FRAMEWIDTH = 50
            FRAMEHEIGHT = 50
            FRAMECOUNTX = 2
            FRAMECOUNTY = 4
            LIGHTMAP = 0
            FRAME = 0 1 1 0 0 1.0 1.0 1.0 1.0 1.0 100
            FRAME = 1 1 1 1 0 1.0 1.0 1.0 1.0 1.0 100
            ...
        }

        SIZE = SMALL 3 6 100 100
        {
            ...
        }
    }
}
```

**字段说明：**
- `SIZE = 大小类型 横帧数 纵帧数 帧宽 帧高` - 定义大小层
- `LAYERNO = 层号 累积时间 总帧数` - 定义动画层数据
- `FRAME = 序号 Z序 可见 资源 X Y 旋转 scaleX scaleY 颜色R 颜色G 颜色B 颜色A 延迟` - 定义帧数据

---

## 参考

- [通用参考文档](00_common_reference.md) - 字节序、数据对齐等通用规范
- [战斗系统文档](02_battle_system.md) - 战斗动画相关
- [怪物层系统文档](07_mob_layer_system.md) - 怪物层扩展系统
