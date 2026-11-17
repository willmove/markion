# Verification — automatic application updates

Date: 2026-11-17 (Asia/Shanghai). Host: Windows x86_64.

## Automated results

| Check | Result |
| --- | --- |
| `cargo check --tests --message-format short` | Passed |
| `cargo test --bin markion app::update::tests -- --test-threads=1` | 21 passed, 0 failed, 1 existing network test ignored |
| `cargo test --workspace` | 2,234 passed, 0 failed, 5 ignored across 51 suites, including doc tests |
| `cargo clippy --workspace --message-format short` | Passed with existing repository lint warnings and the existing `proc-macro-error2` future-compatibility notice |
| `cargo fmt --all -- --check` | Passed |
| `openspec validate add-background-app-updates --strict` | Passed |
| `git diff --check` | Passed |

The workspace run includes storage compatibility tests for absent, invalid, true and false automatic-download values, plus the full preferences round trip. Update tests cover five-hour boundaries, RFC 3339 timezone offsets, invalid/future clocks, the legacy startup override, operation/prompt/ready deduplication, manifest version mismatch, staged-file tampering and cleanup, all eight languages, preference persistence/reset without cache invalidation, Later followed by reopening from the menu, dirty restart rejection, and edits arriving during staged-package reading. Reopening from a window action is deferred until that action releases the GPUI window; the regression test exercises this path.

Ignored workspace tests remain the existing DOCX/PDF export integrations (`test_docx_export_integration`, `export_produces_pdf_bytes_with_pandoc`, `engine_and_fallback_both_produce_openable_packages`) and two external-network tests (`concrete_http_client_fetches_external_image`, `live_github_latest_release_can_be_checked`). They are not counted as passed. The local workspace log is `target/background-update-workspace-tests.log`; the lint log is `target/background-update-clippy.log`.

## Native and distribution limits

This is an implementation and automated-test result, not release-package acceptance. No real signed update was downloaded or installed, no live NSIS upgrade/relaunch was performed, and no five-hour wall-clock soak was run. Scheduling is covered with deterministic time inputs. Test package fixtures are inert byte sequences used only for staging-integrity and consent/dirty-state behavior; they are never executed and do not substitute for Minisign end-to-end verification.

Windows production and test targets compiled on this host. Linux AppImage replacement/relaunch and macOS builds were not run here. The Linux path continues to use cargo-packager-updater's authenticated download and transactional replacement APIs. Before publication, verify real signed Windows and AppImage packages through download, Later, menu reopen, dirty-document refusal, clean restart, signature failure and launch/relaunch failure.

Automatic staging and self-installation require the existing embedded updater public key and supported package type: Windows x86_64 or Linux x86_64 launched as an AppImage. macOS, DEB/RPM, unsupported architectures and keyless development builds check periodically and offer browser downloads. Ready packages are retained only for the current session; routine background discovery failures remain silent and retry at the next interval.
