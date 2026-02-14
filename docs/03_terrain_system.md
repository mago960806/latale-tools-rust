# LaTale 客户端资源加载器 - 地形系统文档

## 概述

本文档描述 LaTale 游戏中的地形资源格式，包括背景图像 (.BGI)、地形属性 (.ATTR)、地形形状 (.FORM) 和地形调色板 (.PAL)。

## 版本信息

| 版本 | 日期 | 变更内容 |
|-----|------|---------|
| 0.1 | 2004/03/31 | 初始版本 |

## 目录

- [背景图像系统 (.BGI)](#背景图像系统-bgi)
- [地形属性系统 (.ATTR)](#地形属性系统-attr)
- [地形形状系统 (.FORM)](#地形形状系统-form)
- [地形调色板系统 (.PAL)](#地形调色板系统-pal)
- [参考](#参考)

---

## 背景图像系统 (.BGI)

### 背景层枚举

背景层标识符用于指定不同的图层类型，支持多个背景层、地形层和前景层。

| 枚举值 | 名称 | 描述 |
|-------|------|------|
| -1 | BG_ID_LAYER_NULL | 空层（无效） |
| 0 | BG_ID_LAYER_HEADER | 头部层 |
| 1 | BG_ID_LAYER_BACKGROUND0 | 背景层 0 |
| 2 | BG_ID_LAYER_BACKGROUND1 | 背景层 1 |
| 3 | BG_ID_LAYER_BACKGROUND2 | 背景层 2 |
| 4 | BG_ID_LAYER_BACKGROUND3 | 背景层 3 |
| 5 | BG_ID_LAYER_TERRAIN0 | 地形层 0 |
| 6 | BG_ID_LAYER_TERRAIN1 | 地形层 1 |
| 7 | BG_ID_LAYER_TERRAIN2 | 地形层 2 |
| 8 | BG_ID_LAYER_FOREGROUND0 | 前景层 0 |
| 9 | BG_ID_LAYER_FOREGROUND1 | 前景层 1 |
| 10 | BG_ID_LAYER_FOREGROUND2 | 前景层 2 |
| 11 | BG_ID_LAYER_FOREGROUND3 | 前景层 3 |

### ASCII 格式规范

```
BGFORMAT_HEADER
{
    BG_ID_LAYER_BACKGROUND0
    {
        fX = 0.0
        fY = 0.0
        fSX = 0.0
        fSY = 0.0
        fDX = 0.0
        fDY = 0.0
        fFlowDX = 0.0
        fFlowDY = 0.0
        fRotateStepX = 0.0
        fRotateStepY = 0.0
        fScaleX = 1.0
        fScaleY = 1.0
        iLightMap = 0
        Image = "background.tga"
    }
}
```

### 二进制格式布局

| 偏移量 | 字段 | 类型 | 大小 | 描述 |
|-------|------|------|------|------|
| 0x00 | fX | float | 4 字节 | X 坐标 |
| 0x04 | fY | float | 4 字节 | Y 坐标 |
| 0x08 | fSX | float | 4 字节 | 滚动速度 X |
| 0x0C | fSY | float | 4 字节 | 滚动速度 Y |
| 0x10 | fDX | float | 4 字节 | 增量 X |
| 0x14 | fDY | float | 4 字节 | 增量 Y |
| 0x18 | fFlowDX | float | 4 字节 | 流动增量 X |
| 0x1C | fFlowDY | float | 4 字节 | 流动增量 Y |
| 0x20 | fRotateX | float | 4 字节 | 旋转 X |
| 0x24 | fRotateY | float | 4 字节 | 旋转 Y |
| 0x28 | fScaleX | float | 4 字节 | 缩放 X |
| 0x2C | fScaleY | float | 4 字节 | 缩放 Y |
| 0x30 | iLightMap | int | 4 字节 | 光照贴图 |
| 0x34 | eBGLayer | int | 4 字节 | 背景层 ID |
| 0x38 | szImageName | char[] | 64 字节 | 图片文件名 |
| **总计** | | | **104 字节** | |

---

## 地形属性系统 (.ATTR)

### 地形属性类型枚举

地形属性定义了不同类型的地形块，影响角色的移动和交互。

| 枚举值 | 名称 | 描述 |
|-------|------|------|
| 0 | TA_NORMAL | 普通地面 |
| 1 | TA_WALL | 墙壁 |
| 2 | TA_PLATFORM | 平台 |
| 3 | TA_HAZARD | 危险区域 |
| 4 | TA_WATER | 水域 |
| 5 | TA_ICY | 冰面 |
| 6 | TA_LADDER | 梯子 |
| 7 | TA_ROPE | 绳子 |

### ASCII 格式规范

```
TERRAINATTRIBUTE_HEADER
{
    COMMONPALLET = ""
    LOCALPALLET = ""

    TERRAINLAYER_CX = 2000
    TERRAINLAYER_CY = 1000

    TERRAINATTRIBUTE_LAYER1
    {
        0 0 0
        10 20 1
        ...
    }

    TERRAINATTRIBUTE_LAYER2
    {
        ...
    }

    TERRAINATTRIBUTE_LAYER3
    {
        ...
    }
}
```

### 二进制格式布局

#### 文件头部

| 偏移量 | 字段 | 类型 | 大小 | 描述 |
|-------|------|------|------|------|
| 0x00 | TERRAINLAYER_CX | int | 4 字节 | 地形层宽度 |
| 0x04 | TERRAINLAYER_CY | int | 4 字节 | 地形层高度 |

#### 属性条目

每个属性条目占据 9 字节：

| 偏移量 | 字段 | 类型 | 大小 | 描述 |
|-------|------|------|------|------|
| 0x00 | iDiffX | int | 4 字节 | X 偏移 |
| 0x04 | iDiffY | int | 4 字节 | Y 偏移 |
| 0x08 | ucType | unsigned char | 1 字节 | 属性类型 |

---

## 地形形状系统 (.FORM)

### ASCII 格式规范

```
TERRAINFORM_HEADER
{
    COMMONPALLET = ""
    LOCALPALLET = ""

    TERRAINLAYER_CX = 2000
    TERRAINLAYER_CY = 1000

    TERRAINFORM_LAYER1
    {
        TILE = 1001 0 0 0 0 1.0 1.0 0
        TILE = 1002 50 0 0 0 1.0 1.0 0
        ...
    }

    TERRAINFORM_LAYER2
    {
        ...
    }

    TERRAINFORM_LAYER3
    {
        ...
    }
}
```

### 二进制格式布局

#### 文件头部

| 偏移量 | 字段 | 类型 | 大小 | 描述 |
|-------|------|------|------|------|
| 0x00 | TERRAINLAYER_CX | int | 4 字节 | 地形层宽度 |
| 0x04 | TERRAINLAYER_CY | int | 4 字节 | 地形层高度 |

#### 瓦片条目

每个瓦片条目占据 40 字节：

| 偏移量 | 字段 | 类型 | 大小 | 描述 |
|-------|------|------|------|------|
| 0x00 | iInstance | INT64 | 8 字节 | 实例 ID |
| 0x08 | fX | float | 4 字节 | X 坐标 |
| 0x0C | fY | float | 4 字节 | Y 坐标 |
| 0x10 | iArg0 | int | 4 字节 | 参数 0 |
| 0x14 | iArg1 | int | 4 字节 | 参数 1 |
| 0x18 | iArg2 | int | 4 字节 | 参数 2 |
| 0x1C | iArg3 | int | 4 字节 | 参数 3 |
| 0x20 | fScaleX | float | 4 字节 | 缩放 X |
| 0x24 | fScaleY | float | 4 字节 | 缩放 Y |

---

## 地形调色板系统 (.PAL)

### 地形模型类型枚举

地形模型定义了不同的地形元素渲染方式。

| 枚举值 | 名称 | 描述 |
|-------|------|------|
| 0 | TMT_STATIC | 静态模型 |
| 1 | TMT_CIRCULAR_MOTION | 循环运动模型 |
| 2 | TMT_FRAME_ANIMATION | 帧动画模型 |

### ASCII 格式规范

#### 静态模型

```
STATICMODEL
{
    INSTANCE = 1001
    IMAGENAME = "tree.tga"
    LIGHTMAP = 0
}
```

#### 循环运动模型

```
CIRCULARMOTION
{
    INSTANCE = 1002
    IMAGENAME = "spinner.tga"
    DELAY = 1000
    RADIAN = 3.14159
    LIGHTMAP = 0
}
```

#### 帧动画模型

```
FRAMEANIMATION
{
    INSTANCE = 1003
    IMAGENAME = "animated.tga"
    DELAY = 100
    MINSTART = 0
    MAXSTART = 3
    LIGHTMAP = 0
    SYNCTILE = 1
    ALPHA = 255
}
```

### 二进制格式布局

#### 文件结构

每个模型条目以 4 字节模型类型开始，值为 -1 表示文件结束。

#### 静态模型 (类型 0)

| 偏移量 | 字段 | 类型 | 大小 | 描述 |
|-------|------|------|------|------|
| 0x00 | model_type | int | 4 字节 | 模型类型 (0) |
| 0x04 | iInstance | INT64 | 8 字节 | 实例 ID |
| 0x0C | iLightMap | int | 4 字节 | 光照贴图 |
| 0x10 | szImageName | char[] | 64 字节 | 图片文件名 |
| **总计** | | | **80 字节** | |

#### 循环运动模型 (类型 1)

| 偏移量 | 字段 | 类型 | 大小 | 描述 |
|-------|------|------|------|------|
| 0x00 | model_type | int | 4 字节 | 模型类型 (1) |
| 0x04 | iInstance | INT64 | 8 字节 | 实例 ID |
| 0x0C | fDelay | float | 4 字节 | 延迟时间 |
| 0x10 | fRadian | float | 4 字节 | 旋转弧度 |
| 0x14 | iLightMap | int | 4 字节 | 光照贴图 |
| 0x18 | szImageName | char[] | 64 字节 | 图片文件名 |
| **总计** | | | **88 字节** | |

#### 帧动画模型 (类型 2)

| 偏移量 | 字段 | 类型 | 大小 | 描述 |
|-------|------|------|------|------|
| 0x00 | model_type | int | 4 字节 | 模型类型 (2) |
| 0x04 | iInstance | INT64 | 8 字节 | 实例 ID |
| 0x0C | fDelay | float | 4 字节 | 延迟时间 |
| 0x10 | fMinStart | float | 4 字节 | 最小起始帧 |
| 0x14 | fMaxStart | float | 4 字节 | 最大起始帧 |
| 0x18 | iLightMap | int | 4 字节 | 光照贴图 |
| 0x1C | iSyncTile | int | 4 字节 | 同步瓦片 |
| 0x20 | iAlpha | int | 4 字节 | 透明度 |
| 0x24 | szImageName | char[] | 64 字节 | 图片文件名 |
| **总计** | | | **100 字节** | |

---

## 参考

- [通用参考文档](00_common_reference.md)
- [关卡系统文档](05_stage_system.md)
