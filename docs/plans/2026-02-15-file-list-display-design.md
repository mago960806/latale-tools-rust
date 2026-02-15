# 文件列表实时显示设计

## 需求

`unpack` 和 `pack` 命令在执行时实时显示正在处理的文件列表，类似于 `unzip` 或 `tar -xvf` 的体验。

## 设计

### 1. 打印格式

动态宽度对齐，根据总数自动调整：

```
  [  1/325] Sprite/Character/00001.dds
  [  2/325] Sprite/Character/00002.dds
```

实现方式：
```rust
let width = total.to_string().len();
println!("  [{:>width$}/{}] {}", current, total, name);
```

### 2. 回调类型

```rust
type FileCallback<'a> = Option<&'a dyn Fn(usize, usize, &str)>;  // (current, total, filename)
```

### 3. API 修改

**SpfReader::unpack**
```rust
pub fn unpack(&self, output_dir: &Path, callback: Option<&dyn Fn(usize, usize, &str)>) -> Result<()>
```

**SpfWriter::write**
```rust
pub fn write(&self, output_path: &Path, callback: Option<&dyn Fn(usize, usize, &str)>) -> Result<()>
```

**SpfWriter::add_from_dir**（移除 verbose 参数）
```rust
pub fn add_from_dir(&mut self, data_dir: &Path, prefix: &str) -> Result<()>
```

### 4. CLI 调用

```rust
let callback = |current: usize, total: usize, name: &str| {
    let width = total.to_string().len();
    println!("  [{:>width$}/{}] {}", current, total, name);
};

// unpack
reader.unpack(output_dir, Some(&callback))?;

// pack
writer.write(&output_path, Some(&callback))?;
```
