<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import {
    Archive,
    ArrowRight,
    BadgeCheck,
    Boxes,
    Check,
    ChevronRight,
    CircleAlert,
    Database,
    FileArchive,
    FileCheck2,
    FileCog,
    FileJson2,
    FileSpreadsheet,
    FolderInput,
    FolderOpen,
    HardDrive,
    Home,
    Info,
    Layers3,
    LoaderCircle,
    LockKeyhole,
    Map,
    Package,
    Search,
    Settings,
    ShieldCheck,
    Table2,
    Wrench,
    X,
  } from "@lucide/svelte";
  import type {
    DatabaseResult,
    LdtInfo,
    OpenRequest,
    OperationResult,
    ProgressEvent,
    RegistryItem,
    SpfInfo,
    StgInfo,
    ViewId,
  } from "./types";

  const navItems = [
    { id: "home" as ViewId, label: "总览", icon: Home },
    { id: "spf" as ViewId, label: "SPF 资源包", icon: Archive },
    { id: "ldt" as ViewId, label: "LDT 数据表", icon: Table2 },
    { id: "stg" as ViewId, label: "STG 地图", icon: Map },
    { id: "pack" as ViewId, label: "单个 SPF 打包", icon: Package },
    { id: "data" as ViewId, label: "DATA 批量打包", icon: Layers3, badge: "未开放" },
  ];

  const encodingOptions = [
    { value: "GBK", label: "GBK（私服）" },
    { value: "BIG5", label: "BIG5（台服）" },
    { value: "EUC-KR", label: "EUC-KR（韩服）" },
    { value: "UTF-8", label: "UTF-8（通用）" },
  ];

  let activeView: ViewId = "home";
  let registries: RegistryItem[] = [];
  let busy = false;
  let errorMessage = "";
  let successMessage = "";
  let progress: ProgressEvent | null = null;

  let spfPath = "";
  let spfInfoData: SpfInfo | null = null;
  let spfSearch = "";
  let spfDatabaseEncoding = "GBK";

  let ldtPath = "";
  let ldtEncoding = "GBK";
  let ldtInfoData: LdtInfo | null = null;

  let stgPath = "";
  let stgEncoding = "GBK";
  let stgInfoData: StgInfo | null = null;

  let packRegistryName = "ROWID";
  let packDataDir = "";
  let packOutputPath = "";
  let packVersion = 0;
  let packEncoding = "GBK";
  let packEncrypted = false;

  let dataRoot = "";
  let selectedBatch = new Set<string>();

  $: selectedRegistry = registries.find((item) => item.name === packRegistryName) ?? null;
  $: filteredSpfFiles = spfInfoData
    ? spfInfoData.files.filter((file) => file.name.toLowerCase().includes(spfSearch.toLowerCase()))
    : [];

  onMount(() => {
    const unlisteners: UnlistenFn[] = [];

    void (async () => {
      registries = await invoke<RegistryItem[]>("spf_registry");
      selectRegistry(packRegistryName);

      unlisteners.push(
        await listen<ProgressEvent>("operation-progress", (event) => {
          progress = event.payload;
        }),
      );
      unlisteners.push(
        await listen<OpenRequest[]>("open-request", (event) => {
          void handleOpenRequests(event.payload);
        }),
      );
      unlisteners.push(
        await getCurrentWebviewWindow().onDragDropEvent((event) => {
          if (event.payload.type === "drop" && event.payload.paths.length > 0) {
            void routePath(event.payload.paths[0]);
          }
        }),
      );

      const requests = await invoke<OpenRequest[]>("launch_requests");
      await handleOpenRequests(requests);
    })().catch(showError);

    return () => unlisteners.forEach((unlisten) => unlisten());
  });

  function setView(view: ViewId) {
    activeView = view;
    clearMessages();
  }

  function clearMessages() {
    errorMessage = "";
    successMessage = "";
  }

  function showError(error: unknown) {
    errorMessage = error instanceof Error ? error.message : String(error);
    successMessage = "";
  }

  async function withTask<T>(task: () => Promise<T>): Promise<T | null> {
    busy = true;
    progress = null;
    clearMessages();
    try {
      return await task();
    } catch (error) {
      showError(error);
      return null;
    } finally {
      busy = false;
    }
  }

  async function pickFile(extensions: string[]): Promise<string | null> {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "支持的文件", extensions }],
    });
    return typeof selected === "string" ? selected : null;
  }

  async function pickDirectory(): Promise<string | null> {
    const selected = await open({ multiple: false, directory: true });
    return typeof selected === "string" ? selected : null;
  }

  async function routePath(path: string) {
    const extension = extensionOf(path);
    if (extension === "spf") {
      activeView = "spf";
      spfPath = path;
      await loadSpfInfo();
    } else if (extension === "ldt" || extension === "csv") {
      activeView = "ldt";
      ldtPath = path;
      ldtInfoData = null;
      if (extension === "ldt") await loadLdtInfo();
    } else if (extension === "stg" || extension === "json") {
      activeView = "stg";
      stgPath = path;
      stgInfoData = null;
      if (extension === "stg") await loadStgInfo();
    } else {
      activeView = "data";
      dataRoot = path;
      successMessage = "已将拖入路径设置为 DATA 根目录";
    }
  }

  async function handleOpenRequests(requests: OpenRequest[]) {
    for (const request of requests) {
      await routePath(request.path);
      if (request.action === "verify" && extensionOf(request.path) === "spf") {
        await verifySpf();
      } else if (request.action === "unpack" && extensionOf(request.path) === "spf") {
        await unpackSpf();
      } else if (request.action === "convert" && ["ldt", "csv"].includes(extensionOf(request.path))) {
        await convertLdt();
      } else if (request.action === "convert") {
        successMessage = extensionOf(request.path) === "stg"
          ? "文件已载入，请点击“导出 JSON”"
          : "文件已载入，请点击“生成 STG”";
      }
    }
  }

  async function browseSpf() {
    const path = await pickFile(["spf", "SPF"]);
    if (path) {
      spfPath = path;
      await loadSpfInfo();
    }
  }

  async function loadSpfInfo() {
    const result = await withTask(() => invoke<SpfInfo>("spf_info", { path: spfPath }));
    if (result) spfInfoData = result;
  }

  async function verifySpf() {
    if (!spfPath) return;
    const issues = await withTask(() => invoke<string[]>("spf_verify", { path: spfPath }));
    if (issues) {
      if (issues.length === 0) successMessage = "验证通过：SPF 文件结构完整";
      else errorMessage = `发现 ${issues.length} 个问题：${issues.slice(0, 3).join("；")}`;
    }
  }

  async function unpackSpf() {
    if (!spfPath) return;
    const result = await withTask(() =>
      invoke<OperationResult>("spf_unpack", { path: spfPath }),
    );
    if (result) successMessage = `${result.summary} · ${result.outputPath}`;
  }

  async function unpackSpfToSqlite() {
    if (!spfPath || spfInfoData?.fileId !== 3) return;
    const result = await withTask(() =>
      invoke<DatabaseResult>("spf_unpack_to_sqlite", {
        path: spfPath,
        encoding: spfDatabaseEncoding,
      }),
    );
    if (!result) return;

    const summary = `已解包 ${result.extractedFiles} 个文件，写入 ${result.importedTables} 张表、${result.importedRows} 行`;
    if (result.failures.length === 0) {
      successMessage = `${summary} · ${result.outputPath}`;
    } else {
      errorMessage = `${summary}；${result.failures.length} 个文件失败：${result.failures.slice(0, 2).join("；")}`;
    }
  }

  async function browseLdt(extensions = ["ldt", "LDT", "csv", "CSV"]) {
    const path = await pickFile(extensions);
    if (!path) return;
    ldtPath = path;
    ldtInfoData = null;
    if (extensionOf(path) === "ldt") await loadLdtInfo();
  }

  async function loadLdtInfo() {
    const result = await withTask(() =>
      invoke<LdtInfo>("ldt_info", { path: ldtPath, encoding: ldtEncoding }),
    );
    if (result) ldtInfoData = result;
  }

  async function convertLdt() {
    if (!ldtPath) return;
    const result = await withTask(() =>
      invoke<OperationResult>("ldt_convert", {
        inputPath: ldtPath,
        encoding: ldtEncoding,
      }),
    );
    if (result) successMessage = `${result.summary} · ${result.outputPath}`;
  }

  async function browseStg() {
    const path = await pickFile(["stg", "STG", "json", "JSON"]);
    if (!path) return;
    stgPath = path;
    stgInfoData = null;
    if (extensionOf(path) === "stg") await loadStgInfo();
  }

  async function loadStgInfo() {
    const result = await withTask(() =>
      invoke<StgInfo>("stg_info", { path: stgPath, encoding: stgEncoding }),
    );
    if (result) stgInfoData = result;
  }

  async function convertStg() {
    if (!stgPath) return;
    const toJson = extensionOf(stgPath) === "stg";
    const outputPath = await save({
      defaultPath: replaceExtension(stgPath, toJson ? ".JSON" : ".STG"),
      filters: [{ name: toJson ? "JSON 文件" : "STG 地图", extensions: [toJson ? "JSON" : "STG"] }],
    });
    if (!outputPath) return;
    const result = await withTask(() =>
      invoke<OperationResult>("stg_convert", {
        inputPath: stgPath,
        outputPath,
        encoding: stgEncoding,
      }),
    );
    if (result) successMessage = `${result.summary} · ${result.outputPath}`;
  }

  function selectRegistry(name: string) {
    packRegistryName = name;
    const registry = registries.find((item) => item.name === name);
    if (registry) {
      packVersion = registry.version;
      packEncoding = registry.encoding;
      if (packOutputPath) packOutputPath = replaceFileName(packOutputPath, `${registry.name}.SPF`);
    }
  }

  async function browsePackData() {
    const path = await pickDirectory();
    if (path) packDataDir = path;
  }

  async function choosePackOutput() {
    const path = await save({
      defaultPath: `${packRegistryName}.SPF`,
      filters: [{ name: "SPF 资源包", extensions: ["SPF"] }],
    });
    if (path) packOutputPath = path;
  }

  async function packSpf() {
    if (!packDataDir || !packOutputPath || !selectedRegistry) {
      errorMessage = "请选择 DATA 根目录和输出文件";
      return;
    }
    const result = await withTask(() =>
      invoke<OperationResult>("spf_pack", {
        spfName: packRegistryName,
        dataDir: packDataDir,
        outputPath: packOutputPath,
        version: packVersion,
        encoding: packEncoding,
        encrypted: packEncrypted,
      }),
    );
    if (result) successMessage = `${result.summary} · ${result.outputPath}`;
  }

  async function browseDataRoot() {
    const path = await pickDirectory();
    if (path) dataRoot = path;
  }

  function toggleBatch(name: string) {
    const next = new Set(selectedBatch);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    selectedBatch = next;
  }

  function extensionOf(path: string) {
    const match = path.match(/\.([^.\\/]+)$/);
    return match ? match[1].toLowerCase() : "";
  }

  function replaceExtension(path: string, extension: string) {
    return path.replace(/\.[^.\\/]+$/, extension);
  }

  function replaceFileName(path: string, fileName: string) {
    const separator = path.includes("\\") ? "\\" : "/";
    const parts = path.split(/[\\/]/);
    parts[parts.length - 1] = fileName;
    return parts.join(separator);
  }

  function fileName(path: string) {
    return path.split(/[\\/]/).pop() || path;
  }

  function encodingLabel(value: string) {
    return encodingOptions.find((option) => option.value === value)?.label ?? value;
  }

  function formatBytes(size: number) {
    if (size >= 1024 ** 3) return `${(size / 1024 ** 3).toFixed(2)} GB`;
    if (size >= 1024 ** 2) return `${(size / 1024 ** 2).toFixed(2)} MB`;
    if (size >= 1024) return `${(size / 1024).toFixed(2)} KB`;
    return `${size} B`;
  }
</script>

<div class="app-shell">
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-mark"><Wrench size={20} strokeWidth={2.4} /></div>
      <div>
        <strong>LaTale Tools</strong>
        <span>资源文件工具</span>
      </div>
    </div>

    <nav>
      <div class="nav-label">工具</div>
      {#each navItems as item}
        <button class:active={activeView === item.id} class="nav-item" onclick={() => setView(item.id)}>
          <item.icon size={18} />
          <span>{item.label}</span>
          {#if item.badge}<small>{item.badge}</small>{/if}
        </button>
      {/each}
    </nav>

    <div class="sidebar-bottom">
      <button class:active={activeView === "settings"} class="nav-item" onclick={() => setView("settings")}>
        <Settings size={18} />
        <span>设置与关于</span>
      </button>
      <div class="version">v0.0.3 · 桌面版</div>
    </div>
  </aside>

  <main class="workspace">
    <header class="topbar">
      <div>
        <div class="eyebrow">资源文件工具</div>
        <h1>{navItems.find((item) => item.id === activeView)?.label ?? "设置与关于"}</h1>
      </div>
      <div class="drop-hint"><FolderInput size={16} /> 拖入文件即可打开</div>
    </header>

    <section class="content">
      {#if activeView === "home"}
        <div class="hero-panel">
          <div>
            <h2>选择要使用的工具</h2>
            <p>也可以将 SPF、LDT、CSV、STG 或 JSON 文件拖入窗口。</p>
          </div>
        </div>

        <div class="tool-grid">
          <button class="tool-card" onclick={() => setView("spf")}>
            <div class="tool-icon red"><FileArchive size={25} /></div>
            <div><h3>SPF 资源包</h3><p>查看、验证和解包 SPF</p></div>
            <ChevronRight size={18} />
          </button>
          <button class="tool-card" onclick={() => setView("ldt")}>
            <div class="tool-icon blue"><FileSpreadsheet size={25} /></div>
            <div><h3>LDT ↔ CSV</h3><p>查看 LDT 数据，双向转换文件</p></div>
            <ChevronRight size={18} />
          </button>
          <button class="tool-card" onclick={() => setView("stg")}>
            <div class="tool-icon amber"><FileJson2 size={25} /></div>
            <div><h3>STG ↔ JSON</h3><p>查看地图结构，双向转换文件</p></div>
            <ChevronRight size={18} />
          </button>
          <button class="tool-card" onclick={() => setView("pack")}>
            <div class="tool-icon graphite"><Package size={25} /></div>
            <div><h3>SPF 打包</h3><p>从 DATA 目录生成 SPF</p></div>
            <ChevronRight size={18} />
          </button>
        </div>

        <div class="home-footer">
          <ShieldCheck size={18} />
          <span>文件只在本机读取和写入。</span>
        </div>
      {:else if activeView === "spf"}
        <div class="page-stack">
          <div class="input-bar">
            <div class="path-box"><FileArchive size={18} /><span>{spfPath || "选择或拖入 SPF 文件"}</span></div>
            <button class="button secondary" onclick={browseSpf}><FolderOpen size={17} />选择文件</button>
          </div>

          {#if spfInfoData}
            <div class="stat-grid five">
              <div class="stat"><span>版本</span><strong>{spfInfoData.version}</strong></div>
              <div class="stat"><span>状态</span><strong class:danger={spfInfoData.encrypted}>{spfInfoData.encrypted ? "已加密" : "未加密"}</strong></div>
              <div class="stat"><span>FILE ID</span><strong>{spfInfoData.fileId}</strong></div>
              <div class="stat"><span>文件数量</span><strong>{spfInfoData.fileCount}</strong></div>
              <div class="stat"><span>包大小</span><strong>{formatBytes(spfInfoData.totalSize)}</strong></div>
            </div>

            <div class="action-row">
              <div class="file-title"><FileArchive size={20} /><div><strong>{fileName(spfInfoData.path)}</strong><span>{spfInfoData.registryName ?? "未注册资源包"} · {spfInfoData.encoding}</span></div></div>
              <button class="button secondary" disabled={busy} onclick={verifySpf}><FileCheck2 size={17} />验证</button>
              <button class="button primary" disabled={busy} onclick={unpackSpf}><FolderInput size={17} />解包</button>
            </div>

            {#if spfInfoData.fileId === 3}
              <div class="database-action-row">
                <div class="database-action-copy"><Database size={20} /><div><strong>生成 latale.db</strong><span>解包后导入包内 LDT，跳过 ID 为 0 的行</span></div></div>
                <select class="compact-select encoding-select" aria-label="LDT 数据编码" title="所有 LDT 使用此编码" bind:value={spfDatabaseEncoding}>{#each encodingOptions as option}<option value={option.value}>{option.label}</option>{/each}</select>
                <button class="button primary" disabled={busy} onclick={unpackSpfToSqlite}><Database size={17} />解包并生成数据库</button>
              </div>
            {/if}

            <div class="table-panel">
              <div class="table-toolbar">
                <strong>包内文件</strong>
                <label class="search"><Search size={15} /><input bind:value={spfSearch} placeholder="搜索路径" /></label>
              </div>
              <div class="file-table">
                <div class="file-row header"><span>#</span><span>资源路径</span><span>大小</span><span>RESID</span></div>
                {#each filteredSpfFiles as file, index}
                  <div class="file-row"><span>{index + 1}</span><span title={file.name}>{file.name}</span><span>{formatBytes(file.size)}</span><span>0x{file.resId.toString(16).padStart(8, "0").toUpperCase()}</span></div>
                {/each}
              </div>
            </div>
          {:else}
            <div class="empty-state"><Archive size={38} /><h3>尚未载入 SPF</h3><p>拖入文件，或点击上方“选择文件”。</p></div>
          {/if}
        </div>
      {:else if activeView === "ldt"}
        <div class="page-stack">
          <div class="input-bar">
            <div class="path-box"><Database size={18} /><span>{ldtPath || "选择 LDT 或 CSV 文件"}</span></div>
            <select class="compact-select encoding-select" aria-label="LDT 文件编码" title="按文件来源选择编码" bind:value={ldtEncoding} onchange={() => ldtInfoData && loadLdtInfo()}>{#each encodingOptions as option}<option value={option.value}>{option.label}</option>{/each}</select>
            <button class="button secondary" onclick={() => browseLdt()}><FolderOpen size={17} />选择文件</button>
          </div>

          <div class="ldt-directions">
            <button class:active={extensionOf(ldtPath) === "ldt"} onclick={() => browseLdt(["ldt", "LDT"])}>
              <div class="direction-formats"><span>LDT</span><ArrowRight size={16} /><span>CSV</span></div>
              <div><strong>导出 CSV</strong><small>选择一个 LDT 文件</small></div>
            </button>
            <button class:active={extensionOf(ldtPath) === "csv"} onclick={() => browseLdt(["csv", "CSV"])}>
              <div class="direction-formats"><span>CSV</span><ArrowRight size={16} /><span>LDT</span></div>
              <div><strong>生成 LDT</strong><small>选择一个 CSV 文件</small></div>
            </button>
          </div>

          {#if ldtPath}
            <div class="conversion-banner">
              <div class="format-pill">{extensionOf(ldtPath).toUpperCase()}</div><ArrowRight size={22} />
              <div class="format-pill target">{extensionOf(ldtPath) === "ldt" ? "CSV" : "LDT"}</div>
              <div class="conversion-copy"><strong>{fileName(ldtPath)}</strong><span>自动保存为同目录下的 {fileName(replaceExtension(ldtPath, extensionOf(ldtPath) === "ldt" ? ".CSV" : ".LDT"))}</span></div>
              <button class="button primary" disabled={busy} onclick={convertLdt}><FileCog size={17} />{extensionOf(ldtPath) === "ldt" ? "导出 CSV" : "生成 LDT"}</button>
            </div>
          {/if}

          {#if ldtInfoData}
            <div class="stat-grid four">
              <div class="stat"><span>数据库 ID</span><strong>{ldtInfoData.databaseId}</strong></div>
              <div class="stat"><span>字段</span><strong>{ldtInfoData.fieldCount}</strong></div>
              <div class="stat"><span>数据行</span><strong>{ldtInfoData.rowCount}</strong></div>
              <div class="stat"><span>文件大小</span><strong>{formatBytes(ldtInfoData.totalSize)}</strong></div>
            </div>
            <div class="table-panel ldt-data-panel">
              <div class="table-toolbar"><strong>全部数据</strong><span>{ldtInfoData.rows.length} 行 · 不分页</span></div>
              <div class="ldt-data-scroll">
                <table class="ldt-data-table">
                  <thead>
                    <tr>
                      <th><span>ID</span><code>int32</code></th>
                      {#each ldtInfoData.fields as field}
                        <th><span>{field.name}</span><code>{field.fieldType}</code></th>
                      {/each}
                    </tr>
                  </thead>
                  <tbody>
                    {#each ldtInfoData.rows as row}
                      <tr>
                        <td>{row.primaryKey}</td>
                        {#each row.values as value}<td>{value}</td>{/each}
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>
          {:else if !ldtPath}
            <div class="empty-state ldt-empty"><Table2 size={34} /><h3>尚未选择文件</h3><p>点击上方“导出 CSV”或“生成 LDT”。</p></div>
          {/if}
        </div>
      {:else if activeView === "stg"}
        <div class="page-stack">
          <div class="input-bar">
            <div class="path-box"><Map size={18} /><span>{stgPath || "选择 STG 或 JSON 文件"}</span></div>
            <select class="compact-select encoding-select" aria-label="STG 文件编码" title="按文件来源选择编码" bind:value={stgEncoding} onchange={() => stgInfoData && loadStgInfo()}>{#each encodingOptions as option}<option value={option.value}>{option.label}</option>{/each}</select>
            <button class="button secondary" onclick={browseStg}><FolderOpen size={17} />选择文件</button>
          </div>

          {#if stgPath}
            <div class="conversion-banner">
              <div class="format-pill">{extensionOf(stgPath).toUpperCase()}</div><ArrowRight size={22} />
              <div class="format-pill target">{extensionOf(stgPath) === "stg" ? "JSON" : "STG"}</div>
              <div class="conversion-copy"><strong>{fileName(stgPath)}</strong><span>{extensionOf(stgPath) === "stg" ? "转换为 JSON" : "转换为 STG"}</span></div>
              <button class="button primary" disabled={busy} onclick={convertStg}><FileCog size={17} />{extensionOf(stgPath) === "stg" ? "导出 JSON" : "生成 STG"}</button>
            </div>
          {/if}

          {#if stgInfoData}
            <div class="stat-grid four">
              <div class="stat"><span>Stage</span><strong>{stgInfoData.stageCount}</strong></div>
              <div class="stat"><span>Group</span><strong>{stgInfoData.groupCount}</strong></div>
              <div class="stat"><span>Map</span><strong>{stgInfoData.mapCount}</strong></div>
              <div class="stat"><span>文件大小</span><strong>{formatBytes(stgInfoData.totalSize)}</strong></div>
            </div>
            <div class="structure-visual">
              <div><Boxes size={28} /><strong>{stgInfoData.stageCount}</strong><span>Stage 容器</span></div>
              <ArrowRight size={22} />
              <div><Layers3 size={28} /><strong>{stgInfoData.groupCount}</strong><span>Map Group</span></div>
              <ArrowRight size={22} />
              <div><Map size={28} /><strong>{stgInfoData.mapCount}</strong><span>Map 节点</span></div>
            </div>
          {:else if !stgPath}
            <div class="empty-state"><Map size={38} /><h3>STG ↔ JSON</h3><p>选择 STG 或 JSON 文件。</p></div>
          {/if}
        </div>
      {:else if activeView === "pack"}
        <div class="pack-layout">
          <div class="form-panel">
            <div class="panel-heading"><Package size={20} /><div><strong>单个 SPF 打包</strong><span>从 DATA 目录生成指定资源包</span></div></div>
            <label class="form-field"><span>资源类型</span><select value={packRegistryName} onchange={(event) => selectRegistry(event.currentTarget.value)}>{#each registries as registry}<option value={registry.name}>{registry.name} · FILE ID {registry.fileId}</option>{/each}</select></label>
            <label class="form-field"><span>DATA 根目录</span><div class="inline-field"><input readonly value={packDataDir} placeholder="选择包含 DATA 文件夹的目录" /><button onclick={browsePackData}><FolderOpen size={17} /></button></div></label>
            <label class="form-field"><span>输出文件</span><div class="inline-field"><input readonly value={packOutputPath} placeholder={`${packRegistryName}.SPF`} /><button onclick={choosePackOutput}><HardDrive size={17} /></button></div></label>
            <div class="form-columns">
              <label class="form-field"><span>版本号</span><input type="number" bind:value={packVersion} /></label>
              <label class="form-field"><span>文件名编码</span><select bind:value={packEncoding}>{#each encodingOptions as option}<option value={option.value}>{option.label}</option>{/each}</select></label>
            </div>
            <label class="toggle-row"><input type="checkbox" bind:checked={packEncrypted} /><span class="toggle"></span><div><strong>加密资源包</strong><small>加密文件数据和索引</small></div></label>
            <button class="button primary wide" disabled={busy || !packDataDir || !packOutputPath} onclick={packSpf}>{#if busy}<LoaderCircle class="spin" size={18} />{:else}<Package size={18} />{/if}开始打包</button>
          </div>

          <aside class="registry-panel">
            <div class="registry-id">{selectedRegistry?.fileId ?? "—"}</div>
            <h3>{selectedRegistry?.name ?? "选择资源类型"}.SPF</h3>
            <p>按此资源类型的目录规则收集文件。</p>
            <div class="registry-meta"><span>默认版本<strong>{selectedRegistry?.version ?? "—"}</strong></span><span>编码<strong>{selectedRegistry ? encodingLabel(selectedRegistry.encoding) : "—"}</strong></span></div>
            <div class="include-list"><span>包含目录</span>{#each selectedRegistry?.includeDirs ?? [] as path}<code>{path}</code>{/each}</div>
          </aside>
        </div>
      {:else if activeView === "data"}
        <div class="page-stack">
          <div class="planned-banner"><Info size={20} /><div><strong>DATA 批量打包</strong><span>此功能暂未开放。</span></div><small>未开放</small></div>
          <div class="input-bar">
            <div class="path-box"><HardDrive size={18} /><span>{dataRoot || "选择游戏 DATA 根目录"}</span></div>
            <button class="button secondary" onclick={browseDataRoot}><FolderOpen size={17} />选择目录</button>
          </div>
          <div class="batch-toolbar"><div><strong>选择要生成的资源包</strong><span>已选择 {selectedBatch.size} / {registries.length}</span></div><div class="batch-actions"><button onclick={() => (selectedBatch = new Set(registries.map((item) => item.name)))}>全选</button><button onclick={() => (selectedBatch = new Set())}>清空</button></div></div>
          <div class="batch-grid">
            {#each registries as registry}
              <button class:selected={selectedBatch.has(registry.name)} class="batch-item" onclick={() => toggleBatch(registry.name)}>
                <span class="check-box">{#if selectedBatch.has(registry.name)}<Check size={14} />{/if}</span>
                <div><strong>{registry.name}.SPF</strong><span>{registry.includeDirs.join(" · ")}</span></div>
                <small>ID {registry.fileId}</small>
              </button>
            {/each}
          </div>
          <div class="batch-footer"><div><Info size={17} /><span>当前版本请使用“单个 SPF 打包”。</span></div><button class="button primary" disabled><Layers3 size={17} />开始批量打包</button></div>
        </div>
      {:else}
        <div class="settings-layout">
          <div class="settings-card"><Wrench size={24} /><h3>LaTale Tools</h3><p>SPF、LDT 和 STG 资源工具。</p><div class="about-list"><span>版本<strong>0.0.3</strong></span><span>支持格式<strong>SPF / LDT / STG</strong></span><span>文件处理<strong>仅限本机</strong></span></div></div>
          <div class="settings-card"><Settings size={24} /><h3>默认行为</h3><label class="setting-row"><span>SPF 打包默认不加密</span><BadgeCheck size={18} /></label><label class="setting-row"><span>解包与 LDT 转换保存到输入文件同目录</span><BadgeCheck size={18} /></label><label class="setting-row"><span>拖入文件后自动选择工具</span><BadgeCheck size={18} /></label></div>
        </div>
      {/if}
    </section>

    <footer class="statusbar">
      <div class="status-message">
        {#if busy && progress}
          <LoaderCircle class="spin" size={15} /><span>{progress.operation} · {progress.current}/{progress.total} · {progress.item}</span>
        {:else if busy}
          <LoaderCircle class="spin" size={15} /><span>正在处理...</span>
        {:else if errorMessage}
          <CircleAlert size={15} /><span class="error-text">{errorMessage}</span><button onclick={() => (errorMessage = "")}><X size={14} /></button>
        {:else if successMessage}
          <BadgeCheck size={15} /><span class="success-text">{successMessage}</span><button onclick={() => (successMessage = "")}><X size={14} /></button>
        {:else}
          <Check size={15} /><span>就绪</span>
        {/if}
      </div>
      {#if progress && busy}<div class="progress-track"><span style={`width: ${(progress.current / Math.max(progress.total, 1)) * 100}%`}></span></div>{/if}
    </footer>
  </main>
</div>
