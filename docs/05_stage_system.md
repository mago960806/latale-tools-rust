# LaTale 关卡系统文档

本文档概述 LaTale STG 关卡/地图数据格式。精确二进制布局见 [STAGENEW.STG 二进制结构说明](STG_BINARY_STRUCTURE.md)。

## 层级结构

```text
Stage
  └─ MapGroup
       └─ MapInfo
```

- `Stage`: 关卡顶层容器。
- `MapGroup`: Stage 下的一组地图，包含连接方式、背景/BGM、物理参数等。
- `MapInfo`: 单张地图的资源引用，包含地形形状、属性和小地图文件。

## 字段命名

Rust JSON 使用 C++ 成员名去掉类型前缀后的 PascalCase：

| C++ 成员 | JSON 字段 |
|---|---|
| `m_iStageID` | `StageID` |
| `m_strStageName` | `StageName` |
| `m_iSyncRegionWidth` | `SyncRegionWidth` |
| `m_iSyncRegionHeight` | `SyncRegionHeight` |
| `m_iGroupID` | `GroupID` |
| `m_iMapLink` | `MapLink` |
| `m_iBGID` | `BGID` |
| `m_iType` | `Type` |
| `m_strGroupName` | `GroupName` |
| `m_iSoundEffectType` | `SoundEffectType` |
| `m_iMiniMapIconID` | `MiniMapIconID` |
| `m_iMiniMapResID` | `MiniMapResID` |
| `m_fGravity` | `Gravity` |
| `m_fMaxDropSpeed` | `MaxDropSpeed` |
| `m_fVelocityX` | `VelocityX` |
| `m_fJumpSpeed` | `JumpSpeed` |
| `m_fUpDownVelocity` | `UpDownVelocity` |
| `m_fHangingVelocity` | `HangingVelocity` |
| `SPMapInfo::iBGIndex` | `BGIndex` |
| `SPMapInfo::strMapName` | `MapName` |

`UnknownStageValue` 和 `UnknownGroupFlag` 是当前二进制文件存在、但当前 `SPStage.h`/`SPStage.cpp` 对象层未命名的字段。

## Stage 字段

| 字段 | 类型 | 说明 |
|---|---|---|
| `StageID` | `int32` | Stage ID。 |
| `SyncRegionWidth` | `int32` | 同步区域宽度。 |
| `SyncRegionHeight` | `int32` | 同步区域高度。 |
| `StageName` | `char[64]` | Stage 名称。 |
| `PaletteFile` | `char[64]` | 地形调色板文件名。 |
| `UnknownStageValue` | `int32` | 当前未命名；`cn` 中恒为 `1000`。 |
| `GroupCount` | `int32` | Group 数量。 |
| `GroupList` | `MapGroup[]` | Group 列表。 |

## MapGroup 字段

| 字段 | 类型 | 说明 |
|---|---|---|
| `GroupID` | `int32` | Group ID。 |
| `MapLink` | `int32` | 地图连接方式，见 `MAP_LINK`。 |
| `BGID` | `int32` | 背景 ID。 |
| `Type` | `int32` | Group 类型位掩码，见 `GROUP_TYPE_*`。 |
| `GroupName` | `char[64]` | Group 名称。 |
| `BGFile` | `char[64]` | BGFormat 文件名。 |
| `BGMFile` | `char[64]` | BGM 文件名。 |
| `SoundEffectType` | `int32` | 音效类型。 |
| `MiniMapIconID` | `int32` | 小地图图标索引；文本字段名为 `ThemeIconIndex`。 |
| `MiniMapResID` | `int32` | 小地图资源 ID；文本字段名为 `ThemeIcon`。 |
| `Gravity` | `int32` | `m_fGravity` 的定点数。 |
| `MaxDropSpeed` | `int32` | `m_fMaxDropSpeed` 的定点数。 |
| `VelocityX` | `int32` | `m_fVelocityX` 的定点数。 |
| `JumpSpeed` | `int32` | `m_fJumpSpeed` 的定点数。 |
| `UpDownVelocity` | `int32` | `m_fUpDownVelocity` 的定点数。 |
| `HangingVelocity` | `int32` | `m_fHangingVelocity` 的定点数。 |
| `UnknownGroupFlag` | `int32` | 当前未命名；`cn` 中只见 `0` 和 `4`。 |
| `MapCount` | `int32` | Map 数量。 |
| `MapList` | `MapInfo[]` | Map 列表。 |

物理参数实际值为 `value * 0.001`。例如 `1000` 表示 `1.0`。

## MapInfo 字段

| 字段 | 类型 | 说明 |
|---|---|---|
| `BGIndex` | `int32` | 背景索引。 |
| `MapName` | `char[64]` | Map 名称。 |
| `FormFile` | `char[64]` | 地形形状文件。 |
| `AttributeFile` | `char[64]` | 地形属性文件。 |
| `MiniMapFile` | `char[64]` | 小地图图片文件。 |

`SPMapInfo::iMapID` 不存盘，旧 C++ 读取时按顺序生成。

## 枚举与位掩码

### MAP_LINK

| 值 | 名称 | 说明 |
|---|---|---|
| 0 | `MAP_LINK_NULL` | 无链接。 |
| 1 | `MAP_LINK_HORIZONTAL` | 横向连接。 |
| 2 | `MAP_LINK_VERTICAL` | 纵向连接。 |

### GROUP_TYPE

| 值 | 名称 |
|---|---|
| `0x001` | `GROUP_TYPE_FIELD` |
| `0x002` | `GROUP_TYPE_MINIMAP` |
| `0x004` | `GROUP_TYPE_EVENT` |
| `0x008` | `GROUP_TYPE_MARKET` |
| `0x010` | `GROUP_TYPE_EXP` |
| `0x020` | `GROUP_TYPE_PVP` |
| `0x080` | `GROUP_TYPE_CASH` |
| `0x100` | `GROUP_TYPE_INDUN` |
| `0x200` | `GROUP_TYPE_REVIVE` |
| `0x400` | `GROUP_TYPE_SUMMON` |

## 字符串与编码

- STG 字符串为固定 `char[64]`。
- 只解码第一个 `\0` 前的有效数据。
- `\0` 后的数据忽略；这对 `tw/STAGENEW.STG` 很重要，因为它可能包含脏填充。
- `cn/STAGENEW.STG` 使用 GBK 严格解码。
- `tw/STAGENEW.STG` 通常使用 Big5。
- JSON 不保存 `Raw` 字段。

## latale-stg CLI

```bash
cargo run --bin latale-stg -- info cn/STAGENEW.STG --stages 2 --groups 2 --maps 1
cargo run --bin latale-stg -- convert cn/STAGENEW.STG -o /tmp/stg
cargo run --bin latale-stg -- convert /tmp/stg/STAGENEW.JSON -o /tmp/stg
```

`info` 输出简洁树，只展示 Stage/Group/Map 的 ID 和名称。名称为空时会用相关文件名作为 fallback。

转换到 JSON 后，`StageCount`、`GroupCount`、`MapCount` 会保留并在写回 STG 时校验。
