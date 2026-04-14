# STAGENEW.STG 二进制结构说明

本文档描述当前 `latale-stg` 使用的 STG 二进制结构。字段命名统一采用 C++ 成员名去掉类型前缀后的 PascalCase，例如 `m_iStageID` 对应 `StageID`，`m_strStageName` 对应 `StageName`。

当前默认目标文件为 `cn/STAGENEW.STG`。`tw/STAGENEW.STG` 结构相同，但字符串编码通常需要使用 Big5，且定长字符串的 `\0` 后可能有脏数据。

## 基本约定

- 字节序：little-endian。
- 整数：`int32`，4 字节，有符号。
- 字符串：固定 `char[64]`，只解码第一个 `\0` 前的数据；`\0` 后的数据忽略。
- `cn/STAGENEW.STG` 默认使用 GBK 严格解码；编码错误应直接报错。
- 文件无魔数、无版本号；首字段就是 `StageCount`。
- JSON 不保存 `Raw` 字段。写回 STG 时按有效字符串重新编码并用 `0x00` 补齐到 64 字节。

## 顶层结构

| 顺序 | 字段 | 类型 | 说明 |
|---|---|---|---|
| 1 | `StageCount` | `int32` | Stage 数量。`cn/STAGENEW.STG` 为 `517`。 |
| 2 | `StageList` | `Stage[StageCount]` | Stage 顺序表。 |

## Stage 结构

| 顺序 | 字段 | 类型 | 说明 |
|---|---|---|---|
| 1 | `StageID` | `int32` | 对应 `SPStage::m_iStageID`。 |
| 2 | `SyncRegionWidth` | `int32` | 对应 `SPStage::m_iSyncRegionWidth`。 |
| 3 | `SyncRegionHeight` | `int32` | 对应 `SPStage::m_iSyncRegionHeight`。 |
| 4 | `StageName` | `char[64]` | 对应 `SPStage::m_strStageName`。 |
| 5 | `PaletteFile` | `char[64]` | 对应 `SPStage::m_vpPalette` 的第一项。 |
| 6 | `UnknownStageValue` | `int32` | 当前 C++ 对象层未命名；`cn` 中恒为 `1000`。 |
| 7 | `GroupCount` | `int32` | 本 Stage 下的 MapGroup 数量。 |
| 8 | `GroupList` | `MapGroup[GroupCount]` | MapGroup 顺序表。 |

说明：`SPStageLoader.cpp` 当前二进制分支在 `PaletteFile` 后直接读取 `GroupCount`，但实际文件需要先读取 `UnknownStageValue`。否则后续字段会错位。

## MapGroup 结构

| 顺序 | 字段 | 类型 | 说明 |
|---|---|---|---|
| 1 | `GroupID` | `int32` | 对应 `SPMapGroup::m_iGroupID`。 |
| 2 | `MapLink` | `int32` | 对应 `SPMapGroup::m_iMapLink`。 |
| 3 | `BGID` | `int32` | 对应 `SPMapGroup::m_iBGID`。 |
| 4 | `Type` | `int32` | 对应 `SPMapGroup::m_iType`，按 `GROUP_TYPE_*` 位掩码解释。 |
| 5 | `GroupName` | `char[64]` | 对应 `SPMapGroup::m_strGroupName`。 |
| 6 | `BGFile` | `char[64]` | 对应 `SPMapGroup::m_strBGFile` 的文件名部分。运行时会补 `DATA/BGFORMAT/` 前缀。 |
| 7 | `BGMFile` | `char[64]` | 对应 `SPMapGroup::m_strBGMFile` 的文件名部分。运行时会补 `DATA/BGM/` 前缀。 |
| 8 | `SoundEffectType` | `int32` | 对应 `SPMapGroup::m_iSoundEffectType`。 |
| 9 | `MiniMapIconID` | `int32` | 对应 `SPMapGroup::m_iMiniMapIconID`，文本字段为 `ThemeIconIndex`。 |
| 10 | `MiniMapResID` | `int32` | 对应 `SPMapGroup::m_iMiniMapResID`，文本字段为 `ThemeIcon`。 |
| 11 | `Gravity` | `int32` | 对应 `SPMapGroup::m_fGravity` 的定点数，实际值为 `value * 0.001`。 |
| 12 | `MaxDropSpeed` | `int32` | 对应 `SPMapGroup::m_fMaxDropSpeed` 的定点数。 |
| 13 | `VelocityX` | `int32` | 对应 `SPMapGroup::m_fVelocityX` 的定点数。 |
| 14 | `JumpSpeed` | `int32` | 对应 `SPMapGroup::m_fJumpSpeed` 的定点数。 |
| 15 | `UpDownVelocity` | `int32` | 对应 `SPMapGroup::m_fUpDownVelocity` 的定点数。 |
| 16 | `HangingVelocity` | `int32` | 对应 `SPMapGroup::m_fHangingVelocity` 的定点数。 |
| 17 | `UnknownGroupFlag` | `int32` | 当前 C++ 对象层未命名；`cn` 中只见 `0` 和 `4`。 |
| 18 | `MapCount` | `int32` | 本组下 Map 数量。 |
| 19 | `MapList` | `MapInfo[MapCount]` | MapInfo 顺序表。 |

说明：`SPStageLoader.cpp` 当前二进制分支读完 6 个物理参数后直接读取 `MapCount`，但实际文件需要先读取 `UnknownGroupFlag`。否则会把 `0` 或 `4` 误读成 `MapCount`。

## MapInfo 结构

| 顺序 | 字段 | 类型 | 说明 |
|---|---|---|---|
| 1 | `BGIndex` | `int32` | 对应 `SPMapInfo::iBGIndex`。 |
| 2 | `MapName` | `char[64]` | 对应 `SPMapInfo::strMapName`。`iMapID` 不存盘，由读取顺序生成。 |
| 3 | `FormFile` | `char[64]` | 对应 `SPMapInfo::strFormFile` 的文件名部分。 |
| 4 | `AttributeFile` | `char[64]` | 对应 `SPMapInfo::strAttributeFile` 的文件名部分。 |
| 5 | `MiniMapFile` | `char[64]` | 对应 `SPMapInfo::strMiniMapFile` 的文件名部分。 |

单个 MapInfo 固定长度为 `4 + 64 * 4 = 260` 字节。

## 枚举与位掩码

`MapLink` 对应 C++ `MAP_LINK`：

| 值 | 名称 | 说明 |
|---|---|---|
| 0 | `MAP_LINK_NULL` | 无链接。 |
| 1 | `MAP_LINK_HORIZONTAL` | 横向连接。 |
| 2 | `MAP_LINK_VERTICAL` | 纵向连接。 |

`Type` 对应 C++ `GROUP_TYPE_*` 位掩码：

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

## JSON 格式

JSON 结构与二进制字段一致：

```json
{
  "StageCount": 517,
  "StageList": [
    {
      "StageID": 0,
      "StageName": "...",
      "GroupCount": 6,
      "GroupList": []
    }
  ]
}
```

写回时会校验 `StageCount`、`GroupCount`、`MapCount` 与实际列表长度一致。

## 工具命令

```bash
cargo run --bin latale-stg -- info cn/STAGENEW.STG --stages 2 --groups 2 --maps 1
cargo run --bin latale-stg -- convert cn/STAGENEW.STG -o /tmp/stg
cargo run --bin latale-stg -- convert /tmp/stg/STAGENEW.JSON -o /tmp/stg
cmp cn/STAGENEW.STG /tmp/stg/STAGENEW.STG
```

`info` 输出只显示一棵简洁树：Stage/Group/Map 的 ID 与名称。名称为空时使用相关文件名作为 fallback。

## 已验证结果

`cn/STAGENEW.STG` 已验证可无损转换：

```text
Stage: 517
MapGroup: 2550
MapInfo: 2983
MD5 (cn/STAGENEW.STG) = dea805c6899ae35dea3818d50cfb9219
MD5 (/tmp/.../STAGENEW.STG) = dea805c6899ae35dea3818d50cfb9219
```
