# Version and platform baseline

This is the supported development baseline for the first A22R version.

| Component | Baseline | Status |
| --- | --- | --- |
| Rust | `1.99.1` stable | pinned by `rust-toolchain.toml` |
| macOS | Ventura `13.5+` | current development baseline |
| Xcode | `15.2` with matching Command Line Tools | required for the Ventura baseline |
| GPUI Kit | `gpui-kit = 0.8.1` | pinned production dependency |
| GPUI pre family | `gpui-pre = 0.3.10` | patched local source copy |
| GPUI source snapshot | `08cbb88a152c3751a43e853425a70e9f631a5918` | recorded by the pinned package metadata |
| Windows | x86_64 Windows 10 22H2+ / Windows 11 | candidate target, not yet verified |

## GPUI policy

The workspace uses one pinned GPUI Kit family. Local source copies under
`forks/gpui-kit` provide the narrow Inspector hooks and overlay guards listed
in the [fork README](../forks/gpui-kit/README.md). The root `Cargo.lock` is
authoritative.

The lockfile may still contain transitive AccessKit crates supplied by the
pinned GPUI accessibility stack. The removed standalone AccessKit candidate
was a separate incompatible prototype and is not a workspace member or
production adapter.

Do not replace the pinned family with floating `main`, `gpui-unofficial`, or an
unverified community fork. Treat the published package API and the selected
source snapshot as one recorded compatibility baseline.

## macOS verification boundary

The current development machine is Darwin 22.x / macOS Ventura. The pinned
application has been exercised for native window creation, GPUI Kit controls,
text input, and native Accessibility smoke on that baseline. The result is
empirical compatibility for this project, not a promise of official support
from every upstream GPUI Kit component.

Manual checks still required for release confidence include native dialogs,
clipboard, IME, focus restoration, VoiceOver, screen-recording-dependent
inspection, restart persistence, and packaging.

## Windows status

The dependency graph contains the x86_64 Windows backend and the repository
includes a Developer PowerShell packaging script. A real Windows machine must
still verify:

- locked workspace build and launch;
- native controls and fonts;
- data persistence after restart;
- packaging and clean-machine launch;
- Accessibility tree and ignored native E2E scenarios.

Until that run is complete, README and release notes must describe Windows as
an unverified candidate target.

## Fonts and licenses

The application bundles Onest and Roboto Flex variable fonts under the SIL Open
Font License 1.1. The corresponding notices are in
`docs/licenses/OFL-1.1-Onest.txt` and
`docs/licenses/OFL-1.1-RobotoFlex.txt`.

## Verification commands

Use the pinned toolchain and lockfile:

```bash
cargo +1.99.1 fmt --all -- --check
cargo +1.99.1 check --workspace --all-targets --locked
cargo +1.99.1 clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo +1.99.1 test --workspace --locked
```

Native E2E is separate because it requires a desktop session and Accessibility
permission. See [testing tools](06-testing-tools.md).
