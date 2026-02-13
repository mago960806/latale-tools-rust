# LaTale 客户端资源加载器 - 特效系统文档

## 概述

本文档描述 LaTale 游戏中的特效资源格式，包括特效组 (.FXG) 和特效模型 (.FXM)。

## 版本信息

| 版本 | 日期 | 变更内容 |
|-----|------|---------|
| 3.2 | 2008/10/06 | 最新版本 |

### 版本历史

| 版本 | 日期 | 变更内容 |
|-----|------|---------|
| 1.0 → 1.1 | - | 添加 Loop, ScreenRender |
| 1.1 → 2.0 | - | 添加纹理效果 |
| 2.0 → 2.1 | - | 帧缩放分离 |
| 2.1 → 3.0 | - | 粒子系统重构 |
| 3.0 → 3.1 | - | 添加模型ID和类型 |
| 3.1 → 3.2 | - | 添加渲染层 |

## 目录

- [特效组系统 (.FXG)](#特效组系统-fxg)
- [特效模型系统 (.FXM)](#特效模型系统-fxm)
- [数据结构定义](#数据结构定义)

---

## 特效组系统 (.FXG)

### 文件格式结构

.FXG 文件用于定义特效组，包含组ID和模型ID的映射关系。

### ASCII 格式

```
FXG_GROUP_ID = 1000
FXG_MODEL_ID = 2000
```

### 二进制格式布局

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | UINT | 4 | uiGroupID | 特效组 ID |
| 0x04 | UINT | 4 | uiModelID | 特效模型 ID |

**总大小: 8 字节**

---

## 特效模型系统 (.FXM)

### 文件格式结构

.FXM 文件用于定义特效模型，支持两种类型：
1. 帧动画特效
2. 粒子特效

### 特效模型类型枚举

| 值 | 名称 | 说明 |
|----|------|------|
| 0 | NULL | 空特效 |
| 1 | FRAMEANIMATION | 帧动画特效 |
| 2 | PARTICLE | 粒子特效 |

### 纹理效果枚举

| 值 | 名称 | 说明 |
|----|------|------|
| 0 | NULL | 无效果 |
| 1 | ALPHA | Alpha 混合 |
| 2 | ADD | 加法混合 |
| 3 | SUBTRACT | 减法混合 |
| 4 | MULTIPLY | 乘法混合 |

### 渲染层枚举

| 值 | 名称 | 说明 |
|----|------|------|
| 0 | OBJECT_FRONT | 物体前层 |
| 1 | OBJECT_BACK | 物体后层 |
| 2 | UI_FRONT | UI 前层 |
| 3 | UI_BACK | UI 后层 |

### ASCII 格式

#### 帧动画特效 (v3.2)

```
FXG_TYPE = 1
FXG_VERSION = 3.2
FXG_ID = 100

FXG_PATH = "effect/fx_name.tga"
FXG_LIFETIME = 2000
FXG_FOLLOW_PARENT = 0

FXG_FXMODEL_ID = 2000
FXG_FXMODEL_TYPE = 1
FXG_FXMODEL_RENDERLAYER = 0

FRAME = 100
{
    TIME = 0
    SCALEX = 1.0
    SCALEY = 1.0
    ROTATION = 0.0
    LRSWAP = 0
    COLOR = 1.0 1.0 1.0 1.0
    SRCRECT = 0 0 256 256
    ADJUSTRECT = 0 0 256 256
}
```

#### 粒子特效 (v3.2)

```
FXG_TYPE = 2
FXG_VERSION = 3.2
FXG_ID = 101

FXG_PATH = "effect/particle.tga"
FXG_LIFETIME = 3000
FXG_FOLLOW_PARENT = 1

FXG_FXMODEL_ID = 2001
FXG_FXMODEL_TYPE = 2
FXG_FXMODEL_RENDERLAYER = 1

AREATYPE = 0
PATHROTATION = 0

FRAMEDATATYPE = 0
VALUEDATATYPE = 0
OBJECTVALUEDATATYPE = 0

FRAME = 0
{
    TIME = 0
    SCALEX = 1.0
    SCALEY = 1.0
    ROTATION = 0.0
    LRSWAP = 0
    COLOR = 1.0 1.0 1.0 1.0
    SRCRECT = 0 0 64 64
    ADJUSTRECT = 0 0 64 64
    AREARECT = 0 0 100 100
    CREATETIME = 100
    CREATECOUNT = 5
    MAGNETPOS = 0 0
    FIRSTKEYTIME = 0
    SECONDKEYVALUE = 0
}
```

### 二进制格式布局

#### 文件头 (SPFXStreamHeader)

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | char[255] | 255 | szDescription | 描述信息 |
| 0xFF | int | 4 | iBinary | 二进制标识 (固定值: 415) |
| 0x103 | float | 4 | fVersion | 版本号 |
| 0x107 | unsigned long | 4 | ulExpansion | 扩展数据 |

**总大小: 267 字节**

#### 默认数据 (SPFXStreamDefaultData, v3.2)

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | TEXTURE_EFFECT | 4 | eTextureEffect | 纹理效果 |
| 0x04 | int | 4 | iFollowParent | 跟随父级标志 |
| 0x08 | float | 4 | fLifeTime | 生命周期 (毫秒) |
| 0x0C | char[255] | 255 | szPath | 图片路径 |
| 0x10B | UINT | 4 | uiFXModelID | FX 模型 ID (v3.1+) |
| 0x10F | SPID_FX_MODEL | 4 | eFXModelType | FX 模型类型 (v3.1+) |
| 0x113 | int | 4 | iFXMRenderLayer | FXM 渲染层 (v3.2+) |

**总大小: 276 字节**

#### 帧数据 (SPFXFrameData)

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | float | 4 | fTime | 时间 (毫秒) |
| 0x04 | float | 4 | fScaleX | X 缩放 (v2.1+ 分离) |
| 0x08 | float | 4 | fScaleY | Y 缩放 (v2.1+ 分离) |
| 0x0C | float | 4 | fRadian | 旋转角度 (弧度) |
| 0x10 | int | 4 | iLRSwap | 左右翻转 (0=否, 1=是) |
| 0x14 | D3DXCOLOR | 16 | Color | 颜色 (RGBA) |
| 0x24 | RECT | 16 | SrcRect | 源矩形 |
| 0x34 | RECT | 16 | AdjustRect | 调整矩形 |

**总大小: 68 字节**

#### 粒子帧数据 (SPFXParticleFrameData)

包含帧数据 (68 字节) 和以下粒子特有字段:

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x44 | RECT | 16 | AreaRect | 区域矩形 |
| 0x54 | float | 4 | fCreateTime | 创建时间 (毫秒) |
| 0x58 | int | 4 | iCreateCount | 创建数量 |
| 0x5C | POINT | 8 | MagnetPos | 磁铁位置 |
| 0x64 | float | 4 | fFirstKeyTime | 第一个关键帧时间 |
| 0x68 | float | 4 | fSecondKeyValue | 第二个关键值 |

**粒子帧数据总大小: 108 字节**

### 历史版本数据结构

#### v1.0 数据结构 (SPFXStreamDefaultData10)

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | float | 4 | fDelayTime | 延迟时间 |
| 0x04 | float | 4 | fLifeTime | 生命周期 |
| 0x08 | char[255] | 255 | szPath | 图片路径 |

**总大小: 263 字节**

#### v1.1 数据结构 (SPFXStreamDefaultData11)

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | bool | 1 | bLightMap | 光照贴图 |
| 0x01 | int | 4 | iFollowParent | 跟随父级 |
| 0x05 | float | 4 | fLifeTime | 生命周期 |
| 0x09 | float | 4 | fDelayTime | 延迟时间 |
| 0x0D | char[255] | 255 | szPath | 图片路径 |

**总大小: 264 字节**

#### v2.0 数据结构 (SPFXStreamDefaultData20)

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | TEXTURE_EFFECT | 4 | eTextureEffect | 纹理效果 |
| 0x04 | int | 4 | iFollowParent | 跟随父级 |
| 0x08 | float | 4 | fLifeTime | 生命周期 |
| 0x0C | float | 4 | fDelayTime | 延迟时间 |
| 0x10 | char[255] | 255 | szPath | 图片路径 |

**总大小: 271 字节**

#### v3.0 数据结构 (SPFXStreamDefaultData30)

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | TEXTURE_EFFECT | 4 | eTextureEffect | 纹理效果 |
| 0x04 | int | 4 | iFollowParent | 跟随父级 |
| 0x08 | float | 4 | fLifeTime | 生命周期 |
| 0x0C | float | 4 | fDelayTime | 延迟时间 |
| 0x10 | char[255] | 255 | szPath | 图片路径 |

**总大小: 271 字节**

#### v3.1 数据结构 (SPFXStreamDefaultData31)

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | TEXTURE_EFFECT | 4 | eTextureEffect | 纹理效果 |
| 0x04 | int | 4 | iFollowParent | 跟随父级 |
| 0x08 | float | 4 | fLifeTime | 生命周期 |
| 0x0C | char[255] | 255 | szPath | 图片路径 |
| 0x10B | UINT | 4 | uiFXModelID | FX 模型 ID |
| 0x10F | SPID_FX_MODEL | 4 | eFXModelType | FX 模型类型 |

**总大小: 272 字节**

---

## 数据结构定义

### 基础数据类型

#### D3DXCOLOR (16 字节)

| 偏移量 | 类型 | 字段名 | 说明 |
|-------|------|--------|------|
| 0x00 | float | r | 红色分量 (0.0-1.0) |
| 0x04 | float | g | 绿色分量 (0.0-1.0) |
| 0x08 | float | b | 蓝色分量 (0.0-1.0) |
| 0x0C | float | a | Alpha 分量 (0.0-1.0) |

#### RECT (16 字节)

| 偏移量 | 类型 | 字段名 | 说明 |
|-------|------|--------|------|
| 0x00 | int | left | 左边界 |
| 0x04 | int | top | 上边界 |
| 0x08 | int | right | 右边界 |
| 0x0C | int | bottom | 下边界 |

#### POINT (8 字节)

| 偏移量 | 类型 | 字段名 | 说明 |
|-------|------|--------|------|
| 0x00 | int | x | X 坐标 |
| 0x04 | int | y | Y 坐标 |

### 粒子特效特有枚举

#### 区域类型 (AreaType)

| 值 | 名称 | 说明 |
|----|------|------|
| 0 | RECTANGLE | 矩形区域 |
| 1 | CIRCLE | 圆形区域 |

#### 路径旋转 (PathRotation)

| 值 | 名称 | 说明 |
|----|------|------|
| 0 | NONE | 无旋转 |
| 1 | CLOCKWISE | 顺时针 |
| 2 | COUNTERCLOCKWISE | 逆时针 |

#### 帧数据类型 (FrameDataType)

| 值 | 名称 | 说明 |
|----|------|------|
| 0 | LINEAR | 线性插值 |
| 1 | BEZIER | 贝塞尔曲线 |

---

## 路径加密

文件路径在 .FXM 文件中使用 XOR 255 加密存储。

### 加密算法

```
加密后字符 = 原字符 XOR 255
解密字符 = 加密字符 XOR 255
```

### 示例

原始路径: "effect/fx_name.tga"

加密过程:
- 'e' (0x65) XOR 255 = 0x9A
- 'f' (0x66) XOR 255 = 0x99
- 'f' (0x66) XOR 255 = 0x99
- 'e' (0x65) XOR 255 = 0x9A
- 'c' (0x63) XOR 255 = 0x9C
- 't' (0x74) XOR 255 = 0x8B
- ...

---

## 参考

- [通用参考文档](00_common_reference.md)
