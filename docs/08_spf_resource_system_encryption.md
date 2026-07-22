# LaTale 新版 SPF 资源加密机制

## 概述

本文记录台服新版 LaTale 客户端对 SPF（Simple Pack File）资源包增加的加密机制。

基础容器结构、`FINFO`、`F_SPF_HEADER`、`RESID` 等定义参见 [SPF 资源系统文档](08_spf_resource_system.md)。新版 SPF 没有改变原有容器布局，而是在资源数据区和索引记录上增加了 ChaCha20 流加密。

本文结论基于以下样本：

| 文件 | 大小 | 版本 | 文件数 | 说明 |
|---|---:|---:|---:|---|
| `ROWID-未加密.SPF` | 175,585,601 字节 | `2026060401` | 367 | 更新前明文样本 |
| `ROWID.SPF` | 179,308,607 字节 | `2026071602` | 370 | 更新后加密样本 |
| `LaTaleClient.exe` | 8,911,872 字节 | - | - | 新版 64 位客户端 |

客户端 SHA-256：

```text
0420CCFE728EBD8BF483FE172948FA52C2A139917F405E4006B6EC29CCACFCA3
```

本文中的函数地址和硬编码参数只保证适用于上述客户端版本。后续客户端更新可能改变地址或加密参数。

## 总体结论

新版 SPF 按区域采用不同处理方式：

| SPF 区域 | 新版处理方式 |
|---|---|
| 资源数据区 | 使用按资源动态派生状态的定制 ChaCha20 加密 |
| `FINFO` 索引表 | 每条记录使用相同参数的标准 ChaCha20 加密 |
| `F_SPF_HEADER` | 保持明文 |
| 末尾版本号 | 最高位作为加密标志，其余位保存原版本号 |

ChaCha20 是流加密，密文长度与明文长度相同，加密和解密执行相同的 XOR 操作：

```text
ciphertext = plaintext XOR keystream
plaintext  = ciphertext XOR keystream
```

因此新版 SPF 不需要额外的填充字节，原有资源偏移和大小语义保持不变。

## 新版文件布局

```text
+-----------------------------+
| 加密的资源数据区             |  <- 偏移 0
+-----------------------------+
| 加密的 FINFO 索引表          |
+-----------------------------+
| 明文 F_SPF_HEADER（136 字节）|
+-----------------------------+
| 带加密标志的版本号（4 字节） |
+-----------------------------+  <- 文件末尾
```

尾部定位方式与旧版完全一致：

```text
版本号位置 = 文件大小 - 4
HEADER 位置 = 文件大小 - 140
索引起点 = 文件大小 - 140 - HEADER.iHeaderSize
资源数据区大小 = 索引起点
```

## 版本号与加密标志

版本字段仍是 4 字节小端无符号整数。新版使用最高位 `0x80000000` 表示该 SPF 的资源和索引已经加密：

```python
encrypted = bool(raw_version & 0x80000000)
version = raw_version & 0x7FFFFFFF
```

新版 `ROWID.SPF` 的磁盘值为：

```text
原始版本值：0xF8C36632
磁盘字节：  32 66 C3 F8
加密标志：  True
显示版本号：0x78C36632 = 2026071602
```

最高位只是状态标志，不是密码学密钥。需要特别注意：资源数据的 ChaCha20 状态派生使用包含最高位标志的原始值 `0xF8C36632`，不能使用清除标志后的 `0x78C36632`。

## F_SPF_HEADER

新版 `F_SPF_HEADER` 仍为明文：

```cpp
struct F_SPF_HEADER {
    int  iHeaderSize;
    int  iFileID;
    char szDesc[128];
};
```

新版 `ROWID.SPF` 的 HEADER：

```text
iHeaderSize = 51800
iFileID     = 3
szDesc      = ""
```

文件数量仍按旧公式计算：

```text
51800 / 140 = 370 条 FINFO
```

## FINFO 索引加密

### 算法

每条 140 字节 `FINFO` 使用标准 20 轮 ChaCha20 独立加解密。客户端会为每条记录重新初始化相同的 key、counter 和 nonce，因此所有 `FINFO` 都复用相同的 140 字节密钥流。

ChaCha20 标准常量为：

```text
"expand 32-byte k"
```

对应状态字：

```text
61707865 3320646E 79622D32 6B206574
```

### FINFO ChaCha20 key

实际 256 位 key 为：

```text
8D C6 10 48 A0 19 AD DE 72 39 EF B7 5F E6 52 21
8D C6 10 48 A0 19 AD DE 72 39 EF B7 5F E6 52 21
```

按 32 位小端状态字表示：

```text
4810C68D DEAD19A0 B7EF3972 2152E65F
4810C68D DEAD19A0 B7EF3972 2152E65F
```

### Counter 与 nonce

```text
counter = 00000000

nonce =
3A 00 00 00
C5 FF FF FF
B7 C6 10 48
```

按状态字表示：

```text
00000000 0000003A FFFFFFC5 4810C6B7
```

### 完整初始状态

```text
61707865 3320646E 79622D32 6B206574
4810C68D DEAD19A0 B7EF3972 2152E65F
4810C68D DEAD19A0 B7EF3972 2152E65F
00000000 0000003A FFFFFFC5 4810C6B7
```

### 加解密流程

```python
for record in encrypted_finfo_records:
    # 每条记录都从相同的初始状态开始，不能延续上一条的 counter。
    plain_record = chacha20_xor(
        data=record,
        key=FINFO_KEY,
        counter=0,
        nonce=FINFO_NONCE,
    )
```

该状态产生的前 140 字节密钥流为：

```text
86 1A A7 D7 F5 ED B2 AD C0 8B 64 A4 C9 D7 DE 1F
05 4D A0 5A 48 32 E0 DC 0E E7 6B E4 84 52 37 8B
EE 73 4F 74 78 B9 DE 41 A2 1D 82 30 44 E2 EE 74
A8 8F D8 2F C7 DD 79 D3 B6 18 30 14 71 9F CE C7
7A 31 65 9A 3A C1 64 07 82 CD 1A 80 2B 97 26 AE
3A 2C 5F D5 34 C5 4B B5 87 AE 59 4C EA B7 8E 2F
DD 8A 22 F8 22 1B 78 6C 2D 64 5C 5E 96 06 DF 9F
B0 FB B5 B1 80 F8 00 35 D0 1F CE BB 63 40 28 32
65 BF 1F 3F 3E 2F CF 24 15 EF 5D D7
```

最后 12 字节分别作用于 `iOffset`、`iSize` 和 `ResID`：

| 字段 | 密钥流 |
|---|---|
| `iOffset` | `65 BF 1F 3F` |
| `iSize` | `3E 2F CF 24` |
| `ResID` | `15 EF 5D D7` |

## 资源数据加密

### 算法特点

每个内嵌资源单独初始化并执行一次 ChaCha20 XOR。资源之间不会延续 counter。初始状态由以下信息动态派生：

- `FINFO.iOffset`
- `FINFO.iSize`
- SPF 原始版本号，包括 `0x80000000` 加密标志
- 客户端硬编码常量

资源加密使用标准 ChaCha20 的 20 轮 quarter-round 核心，但替换了标准的前 4 个常量字。

自定义常量字为：

```text
6E206843 646E6167 20656854 6B636150
```

对应的 16 个原始字节为：

```text
43 68 20 6E 67 61 6E 64 54 68 65 20 50 61 63 6B
```

### 状态派生

以下伪代码等价于客户端 `0x14062F520` 的核心逻辑。所有 64 位运算均按模 `2^64` 截断，所有 32 位运算均按模 `2^32` 截断。

```python
MASK32 = 0xFFFFFFFF
MASK64 = 0xFFFFFFFFFFFFFFFF

base = 0xDEAD19A04810C68D

# 将资源偏移和大小组合为一个 64 位值。
mixed = ((offset & MASK32) << 32) | (size & MASK32)

# 客户端使用有符号扩展后的原始 32 位版本值。
version64 = sign_extend_32(raw_version)
version_mix = (version64 * 0x9E3779B97F4A7C15) & MASK64

# 对 offset/size 组合值进行一次 MurmurHash 风格混合。
mixed ^= ((mixed >> 13) * 0xC6A4A7935BD1E995) & MASK64
mixed &= MASK64

q0 = (mixed ^ version_mix ^ base) & MASK64
q1 = (((version_mix + base) & MASK64) ^ (mixed >> 32)) & MASK64
q2 = ((((mixed << 17) & MASK64) ^ base) + version_mix) & MASK64
q3 = (rol64(base, 32) ^ ((mixed + version_mix) & MASK64)) & MASK64

state = [
    0x6E206843,
    0x646E6167,
    0x20656854,
    0x6B636150,

    # q0、q1、q2、q3 各按两个小端 uint32 拆分，共 8 个状态字。
    low32(q0), high32(q0),
    low32(q1), high32(q1),
    low32(q2), high32(q2),
    low32(q3), high32(q3),

    low32(mixed),
    offset & MASK32,
    (size ^ raw_version) & MASK32,
    (high32(base) + offset) & MASK32,
]
```

`state[12]` 是 ChaCha20 block counter。每处理一个 64 字节块后加一。

### 示例

新版 `ROWID.SPF` 的第一个资源：

```text
文件名：      DATA/LDT/ABUSE_LIST.LDT
offset：      0
size：        14794
raw_version： 0xF8C36632
```

派生出的完整 ChaCha20 初始状态：

```text
6E206843 646E6167 20656854 6B636150
763A8CC8 F5AEEEFD 68A8C734 CC546A6E
4EAA60A7 7F32FED2 1F6073D9 FC5B3EEC
5BD1D05F 00000000 F8C35FF8 DEAD19A0
```

使用该状态对 SPF 偏移 0 开始的 14,794 字节执行 ChaCha20 XOR，结果与旧版 `ROWID-未加密.SPF` 中的 `DATA/LDT/ABUSE_LIST.LDT` 逐字节完全一致。

## 客户端加载流程

新版客户端的 SPF 加载流程如下：

```text
1. 从文件末尾读取 raw_version
   ↓
2. 检查 raw_version 的最高位
   - 最高位为 1：记录该 FILE_ID 的加密状态
   - 使用 BTR 清除最高位，得到显示版本号
   ↓
3. 读取明文 F_SPF_HEADER
   ↓
4. 根据 iHeaderSize 计算 FINFO 数量和索引起点
   ↓
5. 逐条读取 140 字节 FINFO
   ↓
6. 如果该 SPF 已加密，使用固定参数 ChaCha20 解密当前 FINFO
   ↓
7. 建立文件名和 RESID 索引
   ↓
8. 读取资源时，根据 offset、size、raw_version 动态派生 ChaCha20 状态
   ↓
9. 对资源缓冲区原地执行 ChaCha20 XOR
```

## 客户端函数位置

下列虚拟地址只适用于 SHA-256 为 `0420CCFE...CCACFCA3` 的客户端：

| 虚拟地址 | 作用 |
|---|---|
| `0x14062C450` | SPF 初始化、版本读取、HEADER 读取和 FINFO 加载 |
| `0x14062BC00` | 通过 RESID 读取并解密资源 |
| `0x14062BEB0` | 通过文件名读取并解密资源 |
| `0x14062F370` | 构造固定的 FINFO ChaCha20 状态 |
| `0x14062F520` | 根据 offset、size、raw_version 派生资源 ChaCha20 状态 |
| `0x14062F660` | ChaCha20 XOR 核心 |

客户端导入的 Windows CryptoAPI，以及带有 `Cipher_Encrypt`、`Cipher_Decrypt` 文本的函数属于其他校验或网络封包路径，不是 SPF 加密实现。

## 验证结果

### FINFO 验证

使用客户端还原出的 ChaCha20 参数解密新版全部 370 条 `FINFO` 后：

- 文件名均为有效的 `DATA/LDT/*.LDT` 路径；
- `iOffset` 连续；
- `ResID` 的高 8 位均为 `FILE_ID = 3`；
- 最后一个资源的结束位置与索引起点完全一致；
- 新版相对旧版新增 3 个资源：
  - `DATA/LDT/D_POINT_LIST.LDT`
  - `DATA/LDT/D_POINT_MISSION.LDT`
  - `DATA/LDT/D_POINT_TYPE.LDT`

### 资源内容验证

- `ABUSE_LIST.LDT` 共 14,794 字节，解密结果与旧版逐字节完全一致；
- 额外抽样 99 个不同偏移的资源，合计 1,992,709 字节；
- 其中 92 个资源与旧版逐字节完全一致；
- 其余 7 个资源虽然大小相同，但内容在游戏更新后确实发生变化，不属于解密错误。

这些结果同时验证了 ChaCha20 核心、FINFO 参数、资源状态派生公式以及 raw version 的使用方式。

## 安全性说明

这套机制能够阻止旧工具直接读取新版 SPF，但不构成不可逆保护：

- FINFO 对所有记录重复使用完全相同的 key、counter 和 nonce；
- 资源状态只由 SPF 中已有的 offset、size、版本号和客户端硬编码常量派生；
- 不依赖服务器下发或设备绑定的外部秘密；
- 客户端必须包含完整的解密逻辑和常量。

因此，只要实现上述 ChaCha20 初始化和状态派生流程，就可以离线解密和解包新版 SPF。
