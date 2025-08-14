# chip8.rs

RustでCHIP-8エミュレータを実装したプロジェクトです。デスクトップ版とWebブラウザ版の両方に対応しています。

## 特徴

- **デスクトップ版**: ターミナルでのCUI表示
- **Webブラウザ版**: HTML5 CanvasとWebAssemblyを使用
- **適切な速度制御**: CPU 600Hz、タイマー60Hz
- **3つのゲーム**: BRIX（ブロック崩し）、INVADERS（シューティング）、GUESS（数当て）

## 実行方法

### デスクトップ版

```bash
# 実行
cargo run --bin desktop

# または
cargo build --bin desktop
./target/debug/desktop
```

ゲーム選択画面で1-3のキーを押してゲームを選択してください。

### Webブラウザ版

1. **必要なツールのインストール**
   ```bash
   # wasm-packのインストール（初回のみ）
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

2. **WASMビルド**
   ```bash
   # wasm-browserブランチに切り替え
   git checkout wasm-browser
   
   # WASMパッケージをビルド
   RUSTFLAGS="" wasm-pack build --target web --out-dir pkg
   ```

3. **HTTPサーバーを起動**
   ```bash
   # Python 3を使用
   python3 -m http.server 8000 --bind localhost
   
   # または Node.jsのlive-serverを使用
   npx live-server --port=8000
   ```

4. **ブラウザでアクセス**
   
   http://localhost:8000 にアクセスしてゲームを楽しんでください。

## オンラインで遊ぶ

GitHub Pagesでオンライン版を公開しています：

**🎮 [オンラインでプレイ](https://ksera524.github.io/chip8.rs/)**

※ GitHubリポジトリにpushすると自動的にビルド・デプロイされます。

## 操作方法

### キーマッピング

CHIP-8の16進キーパッド（0-F）をQWERTYキーボードにマッピング：

```
CHIP-8キーパッド    QWERTYキーボード
1  2  3  C         1  2  3  4
4  5  6  D    →    Q  W  E  R
7  8  9  E         A  S  D  F
A  0  B  F         Z  X  C  V
```

### ゲーム別操作

#### BRIX（ブロック崩し）
- **Q**: パドルを左に移動
- **E**: パドルを右に移動

#### INVADERS（スペースインベーダー）
- **Q**: 左移動
- **E**: 右移動
- **W**: 弾を発射

#### GUESS（数当てゲーム）
- **1-F**: 数字を入力（16進数）

## 開発情報

### プロジェクト構成

```
src/
├── main.rs           # デスクトップ版のエントリーポイント
├── lib.rs           # Webブラウザ版のエントリーポイント
├── chip8.rs         # CHIP-8 CPU実装
├── display.rs       # 描画トレイト定義
├── keyboard.rs      # キーボード入力トレイト定義
├── web_display.rs   # ブラウザ版Canvas描画
└── web_keyboard.rs  # ブラウザ版キーボード入力
```

### ブランチ

- **main**: デスクトップ版（タイミング制御修正済み）
- **wasm-browser**: Webブラウザ版

### ビルド設定

- デスクトップ版: `cargo run --bin desktop`
- WASM版: `wasm-pack build --target web`

### 自動デプロイ

`wasm-browser`ブランチにpushすると、GitHub ActionsがWASMビルドを実行してGitHub Pagesに自動デプロイします。

`.github/workflows/deploy.yml`で設定されており、以下の処理を行います：
1. Rustツールチェーンのセットアップ
2. wasm-packのインストール
3. WASMパッケージのビルド
4. GitHub Pagesへのデプロイ

## 技術仕様

- **CPU速度**: 600命令/秒
- **タイマー**: 60Hz（DelayタイマーとSoundタイマー）
- **画面解像度**: 64×32ピクセル（Webブラウザ版では10倍拡大）
- **メモリ**: 4KB（0x000-0xFFF）
- **フォントセット**: 0x000-0x04Fに格納
- **プログラム開始アドレス**: 0x200

## ライセンス

このプロジェクトのライセンスについては、LICENSEファイルを参照してください。