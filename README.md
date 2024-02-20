<h1>sqlp</h1>

### 安装依赖

```sh
pnpm install
```

### 启动

```sh
# 桌面端
pnpm tauri:dev
```

```sh
# 浏览器端
pnpm dev
```

### 打包

```sh
# 桌面端
pnpm tauri:build
```

```sh
# 浏览器端
pnpm build
```

### 图标生成
* 修改public文件夹下的app-icon.png图标为实际项目图标，格式为1024x1024px的png，然后执行下面的命令即可一键生成所有平台的icon并放在src-tauri/icons文件夹中
pnpm icon

### 致谢
* [tauri-pure-admin](https://github.com/pure-admin/tauri-pure-admin)
