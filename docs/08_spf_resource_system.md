# LaTale 客户端资源文件 - SPF资源系统文档

## 概述

本文档描述 LaTale 客户端使用的 SPF（Simple Pack File）资源打包格式。SPF 是一个自定义的资源容器格式，用于将多个游戏资源文件（如动画、特效、纹理、音频等）打包成单一文件，实现快速加载、资源保护和版本控制。

SPF 文件采用"头尾颠倒"的存储结构：文件元数据和索引信息存储在文件末尾，实际文件数据存储在文件开头。这种设计允许快速访问文件信息而无需扫描整个文件。

## 版本信息

| 文档版本 | 日期 | 作者 |
|---------|------|------|
| 1.0 | 2026-02-14 | AI Assistant |

## 目录

- [SPF文件格式](#spf文件格式)
  - [文件结构](#文件结构)
  - [二进制布局](#二进制布局)
  - [版本号机制](#版本号机制)
- [数据结构定义](#数据结构定义)
  - [FINFO 结构](#finfo-结构)
  - [F_SPF_HEADER 结构](#f_spf_header-结构)
  - [F_READSTREAM 结构](#f_readstream-结构)
  - [RESID 编码机制](#resid-编码机制)
- [加载机制](#加载机制)
  - [初始化流程](#初始化流程)
  - [文件索引](#文件索引)
  - [数据读取](#数据读取)
  - [缓存管理](#缓存管理)
- [SPF文件列表](#spf文件列表)
  - [资源分类](#资源分类)
  - [文件命名规范](#文件命名规范)
- [相关类和函数](#相关类和函数)
  - [SPResourceBase 类](#spresourcebase-类)
  - [SPStream 类体系](#spstream-类体系)
  - [使用示例](#使用示例)

---

## SPF文件格式

### 文件结构

SPF 文件由三部分组成：

```
+------------------+
|   文件数据区      |  <- 文件开头 (偏移 0)
+------------------+
|   文件索引表      |
+------------------+
|   SPF 文件头      |
+------------------+
|   版本号         |  <- 文件末尾
+------------------+
```

1. **文件数据区**：所有打包的原始文件数据连续存储
2. **文件索引表**：每个文件的元信息（文件名、偏移量、大小、RESID）
3. **SPF 文件头**：索引表大小和描述信息
4. **版本号**：4字节整数，标识 SPF 格式版本

### 二进制布局

#### 从文件末尾开始的布局

| 偏移量（从末尾） | 类型 | 大小 | 字段名 | 说明 |
|-----------------|------|------|--------|------|
| -4 | int | 4 字节 | spfVer | SPF 版本号 (F_SPF_VERSION) |
| -(4 + 头大小) | F_SPF_HEADER | 可变 | spfHeader | SPF 文件头 |
| -(4 + 头大小 + 索引) | FINFO[] | 可变 | 文件索引表 | N 个文件信息结构 |
| 0 ~ (文件大小 - 索引 - 4 - 头大小) | byte[] | 可变 | 文件数据 | 实际的打包文件数据 |

#### 文件定位计算

```
文件数据区大小 = 文件总大小 - sizeof(F_SPF_VERSION) - sizeof(F_SPF_HEADER) - 索引表大小
索引表开始位置 = 文件总大小 - sizeof(F_SPF_VERSION) - sizeof(F_SPF_HEADER) - 索引表大小
SPF头位置 = 文件总大小 - sizeof(F_SPF_VERSION) - sizeof(F_SPF_HEADER)
版本号位置 = 文件总大小 - sizeof(F_SPF_VERSION)
```

### 版本号机制

SPF 文件末尾存储一个 4 字节整数作为版本标识：

```
类型: int (F_SPF_VERSION)
大小: 4 字节
位置: 文件末尾最后 4 字节
```

读取版本号的流程：
1. 定位到文件末尾
2. 向前偏移 4 字节
3. 读取 4 字节整数
4. 验证版本兼容性

---

## 数据结构定义

### FINFO 结构

文件信息结构，用于索引 SPF 中的单个文件。

```cpp
struct FINFO {
    char    szFileName[MAX_RES_NAME];  // 文件名，最大 128 字符
    int     iOffset;                   // 文件在 SPF 中的偏移量
    int     iSize;                     // 文件大小（字节）
    RESID   ResID;                     // 资源 ID
};
```

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | char[] | 128 字节 | szFileName | 文件名（含路径），C 字符串 |
| 0x80 | int | 4 字节 | iOffset | 文件数据在 SPF 中的起始偏移 |
| 0x84 | int | 4 字节 | iSize | 文件数据大小 |
| 0x88 | RESID | 4 字节 | ResID | 资源标识符 |
| **总计** | - | **136 字节** | - | - |

**注意**: `MAX_RES_NAME` 定义为 128，文件名相对于 SPF 内部路径。

### F_SPF_HEADER 结构

SPF 文件头结构，包含索引表信息。

```cpp
struct F_SPF_HEADER {
    int      iHeaderSize;             // 头部大小（即索引表总大小）
    int      iFileID;                 // SPF 文件 ID
    char     szDesc[32];              // 描述信息
};
```

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | int | 4 字节 | iHeaderSize | 索引表总大小（字节） |
| 0x04 | int | 4 字节 | iFileID | SPF 文件 ID |
| 0x08 | char[] | 32 字节 | szDesc | 描述字符串 |
| **总计** | - | **40 字节** | - | - |

**索引表文件数量计算**:
```
文件数量 = iHeaderSize / sizeof(FINFO)
```

### F_READSTREAM 结构

读取流结构，用于缓存已加载的资源数据。

```cpp
struct F_READSTREAM {
    int      iRefCount;                // 引用计数
    int      iSize;                    // 数据大小
    BYTE*    pBuffer;                  // 缓冲区指针
};
```

| 偏移量 | 类型 | 大小 | 字段名 | 说明 |
|-------|------|------|--------|------|
| 0x00 | int | 4 字节 | iRefCount | 当前引用该资源的流数量 |
| 0x04 | int | 4 字节 | iSize | 缓冲区数据大小 |
| 0x08 | BYTE* | 4 字节 | pBuffer | 指向堆分配的数据缓冲区 |
| **总计** | - | **12 字节** (32-bit) | - | - |

### RESID 编码机制

RESID（资源 ID）是一个 32 位整数，由两部分组成：

```cpp
typedef unsigned int   RESID;                    // 32 位资源 ID
typedef unsigned char  RES_FILE_ID;              // SPF 文件 ID (8 位)
typedef unsigned int   RES_INSTANCE_ID;         // 实例 ID (24 位)
```

#### RESID 位布局

```
31          23                      0
+-------------+--------------------+
|  FILE_ID    |   INSTANCE_ID      |
+-------------+--------------------+
    高 8 位          低 24 位
```

| 位范围 | 字段 | 大小 | 取值范围 |
|-------|------|------|----------|
| 31-24 | RES_FILE_ID | 8 位 | 0 ~ 255 |
| 23-0 | RES_INSTANCE_ID | 24 位 | 0 ~ 16,777,215 |

#### RESID 组合与分解

```cpp
// 组合 RESID
RESID resid = (fileID << 24) | instanceID;

// 分解 RESID
RES_FILE_ID fileID = (resid >> 24) & 0xFF;
RES_INSTANCE_ID instanceID = resid & 0xFFFFFF;
```

#### 实例 ID 范围

- 最小值: `0`
- 最大值: `16777215` (0xFFFFFF)
- 单个 SPF 最大文件数: 16,777,216 个

---

## 加载机制

### 初始化流程

SPF 资源系统使用单例模式管理所有 SPF 文件。

```cpp
// 1. 创建资源管理器
SPResourceBase::Create();

// 2. 预留 SPF 文件列表
SPResourceBase::GetInstance()->ReserveInitFile(RESOURCE_FILES2, MAX_RESOURCE_FILE2);

// 3. 初始化并加载所有 SPF 文件
SPResourceBase::GetInstance()->Init(RES_WORK_RESOURCE_PATH);
```

#### Init 方法详细流程

```
1. 遍历所有预留的 SPF 文件名
   ↓
2. 为每个 SPF 创建内存映射文件流 (SPMemoryMappedFileStream)
   ↓
3. 定位到文件末尾，读取版本号
   ↓
4. 定位到文件头位置，读取 F_SPF_HEADER
   ↓
5. 定位到索引表位置，读取所有 FINFO
   ↓
6. 构建两个哈希表:
   - m_hmFInfoListName: 文件名 → FINFO*
   - m_hmFInfoListIID: RESID → FINFO*
   ↓
7. 保存文件流引用用于后续数据读取
```

### 文件索引

SPF 支持两种索引方式：

#### 通过文件名索引

```cpp
bool SPResourceBase::GetStreamData(const char* pszFilename, SPStream** ppStream)
```

流程：
1. 在 `m_hmFInfoListName` 中查找文件名
2. 获取对应的 FINFO 结构
3. 使用 FINFO.ResID 调用 RESID 版本的 GetStreamData

#### 通过 RESID 索引

```cpp
bool SPResourceBase::GetStreamData(RESID iInstanceID, SPStream** ppStream)
```

流程：
1. 检查缓存 `m_hmStreamList` 是否已有该 RESID 的数据
2. 如果有缓存，增加引用计数并返回
3. 如果没有缓存：
   - 从 `m_hmFInfoListIID` 获取 FINFO
   - 从 RESID 提取 FILE_ID，获取对应的 SPF 文件流
   - 定位到 FINFO.iOffset 位置
   - 读取 FINFO.iSize 字节数据
   - 创建 F_READSTREAM 并缓存

### 数据读取

#### 定位并读取文件数据

```cpp
// 伪代码示例
FINFO* pFInfo = m_hmFInfoListIID[iInstanceID];
SPStream* pSPFStream = m_vStreamRegisted[GetResID2FileID(iInstanceID)];

pSPFStream->Seek(pFInfo->iOffset, SEEK_SET);
pSPFStream->Read(pBuffer, pFInfo->iSize);
```

### 缓存管理

资源系统使用引用计数管理缓存：

```cpp
struct F_READSTREAM {
    int iRefCount;    // 引用计数
    int iSize;
    BYTE* pBuffer;
};
```

**引用计数规则**：
- 每次创建 SPManagedStream 时，iRefCount++
- 释放流时，iRefCount--
- 当 iRefCount == 0 时，释放 pBuffer

---

## SPF文件列表

### 资源分类

SPF 文件按功能和资源类型分类：

| SPF 文件名 | 资源类型 | 描述 |
|-----------|---------|------|
| TESTPACK.SPF | 测试 | 测试资源包 |
| HOSHIM.SPF | 角色 | 角色动画和资源 |
| ROWID.SPF | 角色 | 角色资源 |
| MAKO1298.SPF | 角色 | 角色资源 |
| METALGENI.SPF | 角色 | 角色资源 |
| DALBONG.SPF | 角色 | 角色资源 |
| RYUMS.SPF | 角色 | 角色资源 |
| BANX.SPF | 角色 | 角色资源 |
| BARY.SPF | 角色 | 角色资源 |
| CLAIRE.SPF | 角色 | 角色资源 |
| CVOICE.SPF | 音频 | 语音资源 |
| GUSTAV.SPF | 角色 | 角色资源 |
| Cri.SPF | 音频 | 音频资源（CRI Middleware） |
| DURAGON.SPF | 角色 | 角色资源 |
| CLETS.SPF | 角色 | 角色资源 |
| BORORU.SPF | 角色 | 角色资源 |
| JOOX3.SPF | 角色 | 角色资源 |
| BONGSIK.SPF | 角色 | 角色资源 |
| RM.SPF | 地图/关卡 | 地图资源 |
| JJALRAJO.SPF | 特效 | 特效资源 |
| JJANG.SPF | 特效 | 特效资源 |
| BUGLE.SPF | 角色 | 角色资源 |
| CHIRS.SPF | 角色 | 角色资源 |
| LILY.SPF | 角色 | 角色资源 |
| FUYU.SPF | 角色 | 角色资源 |
| JMULRO.SPF | 角色 | 角色资源 |
| CC.SPF | 角色 | 角色资源 |
| CW.SPF | 角色 | 角色资源 |
| JX.SPF | 角色 | 角色资源 |
| PCZ.SPF | 角色 | 角色资源 |
| BJF.SPF | 角色 | 角色资源 |

### 文件命名规范

#### SPF 文件命名

- **格式**: `[名称].SPF`
- **大小写**: 大写扩展名 `.SPF`
- **禁用标记**: 文件名以 `-` 开头表示该 SPF 不加载

#### 示例

```
正常文件:  "HOSHIM.SPF"      → 会被加载
禁用文件:  "-AJJIYA.SPF"     → 不会被加载
```

#### SPF 内部文件路径

SPF 内的文件路径使用相对路径，通常相对于资源根目录：

```
格式: [目录名/文件名.扩展名]
示例:
- "FX/SKILL/FIREBALL.FXM"
- "CHAR/PLAYER/IDLE.SEQ"
- "BGM/TOWN01.MP3"
```

---

## 相关类和函数

### SPResourceBase 类

SPF 资源系统的核心管理类，采用单例模式。

#### 类定义位置

```
文件: SPCore/SPResourceCore/SPResourceBase.h
实现: SPCore/SPResourceCore/SPResourceBase.cpp
```

#### 主要方法

| 方法签名 | 说明 |
|---------|------|
| `static SPResourceBase* Create()` | 创建资源管理器单例 |
| `static SPResourceBase* GetInstance()` | 获取资源管理器实例 |
| `static void Release()` | 释放资源管理器 |
| `bool Init(const char* pszBasePath)` | 初始化并加载所有 SPF 文件 |
| `void ReserveInitFile(const char* apszReserveFiles[], int iNoFile)` | 预留 SPF 文件列表 |
| `bool GetStreamData(const char* pszFilename, SPStream** ppStream)` | 通过文件名获取数据流 |
| `bool GetStreamData(RESID iInstanceID, SPStream** ppStream)` | 通过 RESID 获取数据流 |
| `void ReleaseStreamData(RESID iInstanceID)` | 释放资源数据，减少引用计数 |
| `RESID FilenameToInstance(const char* pszFilename)` | 文件名转换为 RESID |
| `int GetNoSPF()` | 获取已加载的 SPF 文件数量 |
| `F_SPF_VERSION GetSPFVersionByInx(const int iInx)` | 获取指定 SPF 的版本号 |
| `const char* GetSPFNameByInx(const int iInx)` | 获取指定 SPF 的文件名 |

### SPStream 类体系

流接口类体系，用于统一数据读取方式。

#### 类继承结构

```
SPStream (抽象基类)
    ├─ SPFileStream (文件系统流)
    ├─ SPMemoryMappedFileStream (内存映射文件流)
    └─ SPManagedStream (托管内存流)
```

#### SPStream (基类)

```
文件: SPCore/SPResourceCore/SPStream.h
```

| 方法 | 说明 |
|-----|------|
| `virtual bool Valid()` | 检查流是否有效 |
| `virtual void Seek(int iOffset, int iOrigin)` | 定位流位置 |
| `virtual int Read(void* pBuffer, int iSize)` | 读取数据 |
| `virtual int Write(const void* pBuffer, int iSize)` | 写入数据 |
| `virtual void Release()` | 释放流 |

#### SPMemoryMappedFileStream

```
用途: 将 SPF 文件映射到内存，提高读取性能
```

#### SPManagedStream

```
用途: 包装从 SPF 读取的资源数据，管理引用计数
构造: SPManagedStream(BYTE* pBuffer, int iSize, RESID iInstanceID)
```

### 使用示例

#### 基本使用流程

```cpp
// 1. 初始化资源系统
SPResourceBase::Create();
SPResourceBase::GetInstance()->ReserveInitFile(RESOURCE_FILES2, MAX_RESOURCE_FILE2);
SPResourceBase::GetInstance()->Init(RES_WORK_RESOURCE_PATH);

// 2. 加载资源文件
SPStream* pStream = NULL;
const char* strResource = "FX/SKILL/FIREBALL.FXM";

if (SPResourceBase::GetInstance()->GetStreamData(strResource, &pStream)) {
    if (pStream != NULL && pStream->Valid()) {
        // 3. 读取数据
        pStream->Read(&dataHeader, sizeof(DataHeader));

        // 4. 使用数据...
        ProcessData(dataHeader);

        // 5. 释放流
        pStream->Release();
    }
}

// 6. 关闭资源系统
SPResourceBase::Release();
```

#### 通过 RESID 加载

```cpp
// 通过 RESID 直接加载（如果已知 ID）
RESID resid = 0x01000415;  // FILE_ID=1, INSTANCE_ID=0x415

SPStream* pStream = NULL;
if (SPResourceBase::GetInstance()->GetStreamData(resid, &pStream)) {
    // 处理数据...
    pStream->Release();
}
```

#### 检查资源是否存在

```cpp
// 方法1: 通过文件名
RESID resid = SPResourceBase::GetInstance()->FilenameToInstance("FX/EFFECT.EXPLO.FXM");
if (resid != 0) {
    // 资源存在
}

// 方法2: 直接尝试加载
SPStream* pStream = NULL;
if (SPResourceBase::GetInstance()->GetStreamData(filename, &pStream)) {
    // 资源存在且加载成功
    pStream->Release();
} else {
    // 资源不存在
}
```

---

## 参考

- [通用格式规范](00_common_reference.md) - 数据类型、字节序、对齐规则
- [动画系统文档](01_animation_system.md) - SEQ/MOB 动画文件格式
- [战斗系统文档](02_battle_system.md) - ARG/DRG 战斗数据文件格式
- [地形系统文档](03_terrain_system.md) - FORM/PAL/ATTR 地形文件格式
- [特效系统文档](04_fx_system.md) - FXG/FXM 特效文件格式
- [关卡系统文档](05_stage_system.md) - STG 关卡数据文件格式
- [LDT数据库文档](06_ldt_database.md) - LDT 数据库文件格式
- [怪物层系统文档](07_mob_layer_system.md) - 怪物层文件格式
