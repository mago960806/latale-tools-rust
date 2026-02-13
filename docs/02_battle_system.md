# LaTale 客户端资源文件 - 战斗数据格式规范

## 概述

本文档描述 LaTale 游戏中的战斗数据文件格式，包括攻击数据 (.ARG) 和受击数据 (.DRG)。

## 版本信息

| 版本 | 日期 | 更新内容 |
|-----|------|---------|
| 1.0 | 2004/08/12 | 初始版本 |

## 目录

- [文件类型](#文件类型)
- [文件命名规则](#文件命名规则)
- [攻击类型枚举](#攻击类型枚举)
- [攻击数据格式 (.ARG)](#攻击数据格式-arg)
- [受击数据格式 (.DRG)](#受击数据格式-drg)
- [碰撞检测系统](#碰撞检测系统)

---

## 文件类型

| 扩展名 | 类型 | 用途 |
|-------|------|------|
| .ARG | 攻击数据 | 定义角色的攻击框 |
| .DRG | 受击数据 | 定义角色的受击框 |

---

## 文件命名规则

### 攻击数据文件

```
角色: ANI_{动画索引}.ARG
怪物: MOB_{类ID}.ARG
怪物层: MOBLAYER_{类ID}.ARG
```

### 受击数据文件

```
角色: ANI_{动画索引}.DRG
怪物: MOB_{类ID}.DRG
怪物层: MOBLAYER_{类ID}.DRG
```

---

## 攻击类型枚举 (ATTACK_TYPE)

### 枚举值定义

| 值 | 名称 | 字符标识 | 描述 |
|---|------|---------|------|
| 0 | ATTACK_NORMAL | N | 普通攻击 |
| 1 | ATTACK_PIERCING | P | 穿透攻击 |
| 2 | ATTACK_DOWN | D | 击倒攻击 |
| 3 | ATTACK_BOTH | B | 穿透+击倒 |

---

## 攻击数据格式 (.ARG)

### ASCII 格式结构

```
ANI_CHARACTER_ATTACK00
{
    ATTACK_POINT_NUM = 2 100 3.0 500.0 0
    {
        0 50 60 150 180 N
        1 100 80 200 220 P
    }
}
```

**字段说明：**
- `ATTACK_POINT_NUM = 数量 持续时间 累积时间 攻击类型` - 攻击点数据头
- `序号 左 上 右 下 类型` - 攻击框数据

### 二进制格式结构 - 攻击信息

```
偏移量  类型          字段名            描述
0x00    int           AttackType        攻击类型 (0=普通,1=穿透,2=击倒,3=混合)
0x04    int           Index             索引
0x08    RECT          AttackRange       攻击范围 {left, top, right, bottom}
```

### 二进制格式结构 - 角色攻击数据

```
偏移量  类型          字段名            描述
0x00    unsigned int  AnimationIndex    动画索引 (ANIMATION_INDEX)
0x04    int           AttackPointNum    攻击点数量
0x08    float         CallTickTime      持续时间
0x0C    float         AccumulateTime    累积时间
0x10    int           AttackType        攻击类型
0x14    int           InfoSize          攻击信息数量

对于每个攻击信息:
  0x00    int           AttackType        攻击类型
  0x04    int           Index             索引
  0x08    RECT          AttackRange       攻击范围
    +0x00  int           Left              左边界
    +0x04  int           Top               上边界
    +0x08  int           Right             右边界
    +0x0C  int           Bottom            下边界
```

### 二进制格式结构 - 怪物攻击数据

```
偏移量  类型            字段名            描述
0x00    unsigned int    ClassID           类 ID
0x04    int             AnimationNum      动画数量

对于每个动画:
  0x00    ANIMATION_INDEX  AnimationIndex    动画索引
  0x04    int             AttackPointNum    攻击点数量
  0x08    float           CallTickTime      持续时间
  0x0C    float           AccumulateTime    累积时间
  0x10    int             AttackType        攻击类型
  0x14    int             InfoSize          攻击信息数量

  对于每个攻击信息:
    0x00    int             AttackType        攻击类型
    0x04    int             Index             索引
    0x08    RECT            AttackRange       攻击范围
```

---

## 受击数据格式 (.DRG)

### ASCII 格式结构

#### 角色受击数据

```
ANI_CHARACTER_STAND
{
    40 500 2
    {
        0 0 10 10.0 0
        1 10 20 10.0 0
    }
}
```

**字段说明：**
- `总帧数 累积时间 数据数量` - 受击帧数据头
- `序号 受击类型 延迟时间 累积时间 大小类型` - 角色受击数据

#### 怪物受击数据

```
ANI_MONSTER_STAND
{
    40 500 2
    {
        0 0 50 60 10.0 0
        1 10 100 80 10.0 0
    }
}
```

**字段说明：**
- `总帧数 累积时间 数据数量` - 受击帧数据头
- `序号 受击类型 左 上 右下 延迟时间 大小类型` - 怪物受击数据

### 二进制格式结构 - 受击信息

```
偏移量  类型          字段名            描述
0x00    int           BeAttackedIndex    受击索引
0x04    float         AccumulatedTickTime 累积时间
0x08    int           Index             索引
0x0C    RECT          Range             受击范围
```

### 二进制格式结构 - 角色受击数据

```
偏移量  类型          字段名            描述
0x00    unsigned int  AnimationIndex    动画索引 (ANIMATION_INDEX)
0x04    int           TotalFrame        总帧数
0x08    float         AccumulateTime    累积时间
0x0C    int           InfoSize          受击信息数量

对于每个受击信息:
  0x00    int           BeAttackedIndex    受击索引
  0x04    float         AccumulatedTickTime 累积时间
  0x08    int           Index             索引
  0x0C    RECT          Range             受击范围
```

### 二进制格式结构 - 怪物受击数据

```
偏移量  类型            字段名            描述
0x00    unsigned int    ClassID           类 ID
0x04    int             AnimationNum      动画数量

对于每个动画:
  0x00    ANIMATION_INDEX  AnimationIndex    动画索引
  0x04    int             TotalFrame        总帧数
  0x08    float         AccumulateTime    累积时间
  0x0C    int             InfoSize          受击信息数量

  对于每个受击信息:
    0x00    int             BeAttackedIndex    受击索引
    0x04    float         AccumulatedTickTime 累积时间
    0x08    int             Index             索引
    0x0C    int             MonsterSize       怪物大小 (MONSTER_INDEX)
    0x10    RECT            Range             受击范围
```

---

## 碰撞检测系统

### 碰撞单位

```
RECT_SIZE = 35
```

所有碰撞框的单位是 RECT_SIZE = 35 像素。

### 怪物大小索引 (MONSTER_INDEX)

| 值 | 名称 | 碰撞高度 (像素) |
|---|------|-----------------|
| 0 | MON_TINY | 2 × 35 = 70 |
| 1 | MON_SMALL | 1 × 35 = 35 |
| 2 | MON_MED | 2 × 35 = 70 |
| 3 | MON_BIG | 3 × 35 = 105 |

### 怪物高度计算公式

```
if (monSize == MON_TINY)
    height = 2 × RECT_SIZE
else
    height = monSize × RECT_SIZE
```

### RECT 结构

```
偏移量  类型          字段名            描述
0x00    int           Left              左边界
0x04    int           Top               上边界
0x08    int           Right             右边界
0x0C    int           Bottom            下边界
```

### 碰撞检测

1. **攻击检测**: 角色的攻击框 (ATTACKINFO) 与怪物的受击框 (APCBEATTACKEDINFO) 相交
2. **受击检测**: 怪物的攻击框与角色的受击框相交
3. **坐标系统**: 所有坐标相对于角色/怪物的中心点

---

## 二进制格式布局总结

### 攻击数据 (.ARG) 完整布局

#### 角色攻击数据

```
文件头:
  偏移量  类型          字段名
  0x00    unsigned int  AnimationIndex
  0x04    int           AttackPointNum
  0x08    float         CallTickTime
  0x0C    float         AccumulateTime
  0x10    int           AttackType
  0x14    int           InfoSize

  对于每个攻击信息:
    0x00    int           AttackType
    0x04    int           Index
    0x08    RECT          AttackRange
```

#### 怪物攻击数据

```
文件头:
  偏移量  类型            字段名
  0x00    unsigned int    ClassID
  0x04    int             AnimationNum

  对于每个动画:
    0x00    ANIMATION_INDEX  AnimationIndex
    0x04    int             AttackPointNum
    0x08    float           CallTickTime
    0x0C    float           AccumulateTime
    0x10    int             AttackType
    0x14    int             InfoSize

    对于每个攻击信息:
      0x00    int             AttackType
      0x04    int             Index
      0x08    RECT            AttackRange
```

### 受击数据 (.DRG) 完整布局

#### 角色受击数据

```
文件头:
  偏移量  类型          字段名
  0x00    unsigned int  AnimationIndex
  0x04    int           TotalFrame
  0x08    float         AccumulateTime
  0x0C    int           InfoSize

  对于每个受击信息:
    0x00    int           BeAttackedIndex
    0x04    float         AccumulatedTickTime
    0x08    int           Index
    0x0C    RECT          Range
```

#### 怪物受击数据

```
文件头:
  偏移量  类型            字段名
  0x00    unsigned int    ClassID
  0x04    int             AnimationNum

  对于每个动画:
  0x00    ANIMATION_INDEX  AnimationIndex
  0x04    int             TotalFrame
  0x08    float         AccumulateTime
  0x0C    int             InfoSize

  对于每个受击信息:
    0x00    int             BeAttackedIndex
    0x04    float         AccumulatedTickTime
    0x08    int             Index
    0x0C    int             MonsterSize
    0x10    RECT            Range
```

---

## ASCII 格式规范

### 攻击数据 ASCII 格式

```
ANI_{动画索引}
{
    ATTACK_POINT_NUM = 数量 持续时间 累积时间 攻击类型
    {
        序号 左 上 右 下 类型
        ...
    }
}
```

### 受击数据 ASCII 格式

```
ANI_{动画索引}
{
    总帧数 累积时间 数据数量
    {
        序号 受击类型 延迟时间 累积时间 大小类型
        ...
    }
}
```

**字段类型说明：**
- 攻击类型: N(普通), P(穿透), D(击倒), B(混合)
- 受击类型: 0(站立), 1(受击), 2(击倒)...

---

## 参考

- [通用参考文档](00_common_reference.md) - 字节序、数据对齐等通用规范
- [动画系统文档](01_animation_system.md) - 动画索引定义
- [怪物层系统文档](07_mob_layer_system.md) - 怪物层战斗数据
