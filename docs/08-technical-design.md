# 08. 技術設計

## 8.1 モジュール構成
| モジュール | 役割 |
| --- | --- |
| `hex_editor::app` | アプリケーションライフサイクル管理、モード遷移 |
| `hex_editor::buffer` | メモリマップ + ページングするバッファ管理 |
| `hex_editor::view` | TUI レイアウト、描画、入力処理 |
| `hex_editor::commands` | CLI サブコマンドと操作ロジック |
| `hex_editor::diff` | 差分生成とパッチ適用 |
| `hex_editor::io` | ファイル読み書き、バックアップ、ジャーナル |

## 8.2 データ構造
- **PagedBuffer**: 4 KB ページ単位で mmap/読み込みし、LRU でキャッシュ。大容量でもメモリ効率を確保。
- **ChangeSet**: `{offset, original_bytes, new_bytes, timestamp}` のリスト。Undo/Redo は ChangeSet スタックを双方向リストで管理。
- **Bookmark**: `HashMap<String, u64>` でラベルとオフセットを管理。

## 8.3 永続化
- 保存時は `*.bak` に現行ファイルをコピー → 本体更新 → 成功後に `.bak` を削除。
- ジャーナルファイル（`<file>.journal`）に未保存の ChangeSet を追記し、クラッシュ時に再適用。

## 8.4 ライブラリ選定
- Rust で TUI の Hex Editor を使うために使われるライブラリ
- 描画/UI
  - ratatui
  - crossterm
- バッファ/ファイルアクセス
  - memmap2
  - memmapx
- バイナリ操作
  - bytes
  - bitvec
  - scroll
  - zerocopy
  - hex
  - pretty-hex
  - data-encoding
- Error Handlin/Logging
  - anyhow
  - eyre
- User Settings/Persistent Settings
  - serde + toml / ron
  - bincode

## 8.5 GUI ライブラリ選定
- **egui / eframe**  
  - メリット: 即時モードで実装が簡潔、`App` 状態をそのまま流用しやすく、ホットリロード的な開発も容易。  
  - デメリット: `wgpu`/`glow` を利用するためバイナリサイズがやや大きくなる。  
  - Star: 約 19k (`emilk/egui`)
