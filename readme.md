# LJX High Speed Communication Test

LJ-X8000A の高速通信機能の検証用プロジェクト

## usage

./vendor フォルダに"LJX8_IF.lib"をおいてコンパイル

生成されたバイナリの同一フォルダに"LJX8_IF.dll"を設置 P する

## bin/run_ljx.rs

LJX8000A と高速データ通信 ⇒ データをファイルに保存。

通信のタイミングはコマンドラインで制御する

## 実装メモ

### tokio ランタイムの限定的な使用

LJX の C ライブラリのコールバック関数に Rust の関数を渡している。

データをメッセージ送信で Rust に送信したいが C に非同期関数を渡すことができない。

解決方法を思いつかなかったので await を行わなくてよい std::sync::mpsc::Sender を使っている

## examples

検証用のコード置き場
