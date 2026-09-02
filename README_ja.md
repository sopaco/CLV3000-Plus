<div align="center">
    <img height="320" src="./assets/icons/icon_app.png">

# CLV3000 Plus

**作業はエージェントに、ディスク容量は自分に。**

<a href=".terrain/human"><img alt="Litho Docs" src="https://img.shields.io/badge/Litho-Docs-green?logo=Gitbook&color=%23008a60"/></a>
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
<a href="https://github.com/sopaco/CLV3000-Plus/releases"><img alt="App Download" src="https://img.shields.io/badge/Download-Secure-blue?logo=Download&color=%23008a60"/></a>

<p>
  <a href="./README_zh.md">简体中文</a> ·
  <a href="./README.md">English</a> ·
  <b>日本語</b>
</p>

</div>

WorkBuddy、Deepseek Harness、Codex、Claude Code、Cursor といった AI ツールを、きっとたくさん使ってきたはずです。  
そして「そのときは役に立ちそうだったけれど、その後二度と開かなかった」プロジェクトフォルダも、かなり溜まっているのではないでしょうか。キャッシュ、依存関係、エージェントが残した痕跡を抱えたまま、気づかないうちに数十 GB を食いつぶしています。

CLV3000 Plus は **Windows / macOS** に対応したデスクトップ清理ツールです。**さまざまなエージェントを使っている技術者・非技術者の方**のために、この散らかった状況を整理します。何を消してよいか、どれだけ容量を使っているか、そして削除前にしっかり中身を確認できるように。

### 軽量・グリーン、そして驚くほど高速

無駄な肥大化なし、PC を重くしません。CLV3000 Plus は **Codex と同系統**の高性能 **Rust** 技術で構築されており、動作は極めて高速、CPU とメモリの消費は最小限。数年前の古い PC でも、起動・スキャン・清理のすべてが驚くほど軽快に動きます。

| | | |
|:---:|:---:|:---:|
| <img src="assets/snapshots/snapshot_operation_center.webp" width="280" alt="メイン画面"> | <img src="assets/snapshots/snapshot_aigc_cleaner.webp" height="180" alt="エージェント生成物の清理"> | <img src="assets/snapshots/snapshot_clean.webp" height="180" alt="ストレージ最適化"> |
| メイン画面 / ダッシュボード | エージェント生成物の清理 | ストレージ最適化 |

| | | |
|:---:|:---:|:---:|
| <img src="assets/snapshots/snapshot_theme_cherry.webp" height="180" alt="桜"> | <img src="assets/snapshots/snapshot_theme_aurora.webp" height="180" alt="オーロラ"> | <img src="assets/snapshots/snapshot_theme_neon.webp" height="180" alt="ネオン"> |
| テーマ - 桜 | テーマ - オーロラ | テーマ - ネオン |

---

## できること

- **エージェントが残した試作プロジェクトを発見：** スキャン後、**プロジェクト名・使用容量・検出理由・最後に触れてからの期間**が一覧で表示されます。ひと目で把握できるので、残すか消すかを自分で判断できます。
- **キャッシュと依存関係を安全に清理：** エージェントのプロジェクトだけでなく、各技術スタックのビルドキャッシュや依存ディレクトリ（`node_modules`、`target` など）も検出します。
- **その他の便利な機能**
  - **ワンクリック診断：** よく使うディレクトリを一括スキャンし、解放できる容量をまとめて表示
  - **スタートアップ項目の管理：** 起動時の負荷を軽減（macOS / Windows）
  - **プロセスビューア：** メモリを食いつぶしているプログラムを特定

---

## シンプルモード vs エキスパートモード

| | シンプルモード（推奨） | エキスパートモード |
|---|----------------------|-------------------|
| 対象ユーザー | 日常的に使う方、非技術系の方 | 開発者、細かく制御したい方 |
| 説明のしかた | すべてをやさしい言葉で説明 | 完全なパスと技術的な詳細を表示 |
| 初期選択 | 「安全な」項目のみ | より多くの項目を選択可能 |
| 切り替え | 設定画面でいつでも変更可 | 設定画面でいつでも変更可 |
