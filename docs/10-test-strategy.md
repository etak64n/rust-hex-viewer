# 10. テスト戦略

1. **単体テスト**: PagedBuffer の読み書き、ChangeSet、検索アルゴリズム。
2. **統合テスト**: CLI コマンドを `assert_cmd` で検証。差分生成やパッチ適用の往復確認。
3. **スナップショットテスト**: ratatui の画面描画を `insta` 等で比較。
4. **ベンチマーク**: `criterion` で 10 MB, 50 MB ファイルの読み込み/検索性能を測定。
5. **プロパティテスト**: 任意バイト列に対し `patch(write(read(file))) == file` を QuickCheck 的に検証。
