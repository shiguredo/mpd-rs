# apply_patch 経路の再帰的スタックオーバーフロー（DoS）

- Priority: High
- Created: 2026-07-08
- Completed:
- Model: opencode-zen/hy3-free
- Branch: feature/fix-recursive-stack-overflow-apply-patch

## 目的

信頼できない入力（ライブ MPD のパッチ適用経路）で深いネスト XML を与えられた際、再帰的なシリアライズ/ナビゲーションによりスタックオーバーフロー → プロセス `abort`（DoS）する脆弱性を解消するため。

## 優先度根拠

- スタックオーバーフローは Rust では `panic` ではなく `abort`（プロセス終了）になるため、サービスの可用性を直接的に損なう。
- ライブ MPD の Patch は外部から供給される可能性があり、攻撃面になる。

## 現状

- `src/patch.rs:279` 付近の `write_dom_element` が深度上限なしで自己再帰（`for child in &elem.children { ... write_dom_element(...) }`）。
- `src/patch.rs:448` 付近の `navigate_xpath` も深度上限なしで自己再帰（`navigate_xpath(child_elem, ...)`）。
- xml-rs の `EventReader` は要素ネストを反復（Vec）でパースするため、数万レベルの深いネスト MPD でも `parse` を通る（実証: depth=50000 をパース成功）。
- その後 `apply_patch`（`src/patch.rs:752-767`）が `write()` → `parse_xml_to_dom` → `serialize_dom` → `write_dom_element` と往復し、再帰でスタック消費が蓄積する。`write_dom_element` はフレームあたり `EventWriter::write()` 呼び出しとローカル変数を伴うためスタック消費が大きく、数千〜数万レベルのネストでオーバーフローする可能性が高い。

## 設計方針

- `write_dom_element` と `navigate_xpath` の再帰を、明示的スタックを用いた反復処理に書き直す。
- あるいは深度上限（例: 256 または 1024）を設け、超過時に `Error`（例: `ErrorKind::UnexpectedStructure` または新設の `ErrorKind`）を返して `abort` を防ぐ。

## 完了条件

- 深いネスト XML（例: depth=100000）を含む MPD に `apply_patch` を適用してもプロセスが `abort` しないこと（適切に `Err` を返すか、正常に処理されること）。
- `tests/test_patch.rs`（または `pbt/tests/prop_patch.rs`）で深いネストを用いた回復力テストを追加すること。
- `fuzz/fuzz_targets/fuzz_parse.rs` に `apply_patch` / `parse_patch` を含め、fuzzing でスタックオーバーフローが検出されないことを確認すること。

## 解決方法

- `write_dom_element` / `navigate_xpath` を反復化するか深度打ち切りを入れる。
- fuzz ターゲットを拡張し、パッチ経路も含めてパニック/abort 安全性を継続的に検証する。
