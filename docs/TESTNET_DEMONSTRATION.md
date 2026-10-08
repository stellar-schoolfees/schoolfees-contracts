# schoolfees synthetic testnet demonstration

Deployed October 8, 2026. Reverified `2026-10-08T18:52:08.061Z`.
The maintainer explicitly authorized this demonstration using synthetic data.
No real pilot, partner agreement, security audit or production readiness is claimed.

- Network: Stellar **testnet**, protocol 29 at verification.
- Contract ID: `CBRSL5YIUN3ANQVEJB4YWQAXBN6KMZR55OWBDZS7ZLHCRKLDOOYWAXLR`.
- [Stellar Expert contract](https://stellar.expert/explorer/testnet/contract/CBRSL5YIUN3ANQVEJB4YWQAXBN6KMZR55OWBDZS7ZLHCRKLDOOYWAXLR).
- [Stellar Lab contract explorer](https://lab.stellar.org/r/testnet/contract/CBRSL5YIUN3ANQVEJB4YWQAXBN6KMZR55OWBDZS7ZLHCRKLDOOYWAXLR).
- Source commit: `d664c3827c01144652ac59fe74659a2a2376171b`; runtime source matched the rebuilt artifact. The worktree also contains contributor documentation changes.
- Wasm SHA-256: `016df1942fa7f02a9a2402bc0854d75c0393ba3ae6d39b2de858d040b996afbb`. Local and deployed hashes matched.
- Public deployer account: `GBHVPV4S3JRAOPQYODNULNPBW57REIJ2SDNAHYHN6SXGOKJCS3XLPVZD`. Signing material stays outside Git.

| Transaction | Hash / explorer | RPC verification |
|---|---|---|
| creation | [bacbfdb35ec09b3500d9c094232c0351ac83c75de7037535071c5722041f03cd](https://stellar.expert/explorer/testnet/tx/bacbfdb35ec09b3500d9c094232c0351ac83c75de7037535071c5722041f03cd) | SUCCESS; ledger 5078355 |
| upload | [718152bb7629d1ac9a004531ef6c36e441ba3ef667a5888161100c879a686d53](https://stellar.expert/explorer/testnet/tx/718152bb7629d1ac9a004531ef6c36e441ba3ef667a5888161100c879a686d53) | SUCCESS; ledger 5078353 |
| initialization | [d7d64dbf32af381ed08cfc0b62d387e35842ec34374d39c1b4ee02cfdd1138e0](https://stellar.expert/explorer/testnet/tx/d7d64dbf32af381ed08cfc0b62d387e35842ec34374d39c1b4ee02cfdd1138e0) | SUCCESS; ledger 5078358 |

## Observed checks

Before deployment, formatting, locked clippy with warnings denied, locked Rust tests,
Node checker tests, error/documentation checks and the Stellar CLI 28.1.0 Wasm build passed.
34 Rust tests and 9 Node tests.
During verification, all listed transactions were `SUCCESS` through the testnet RPC
and the deployed Wasm hash matched the tested local artifact.

SchoolFees initialization succeeded; a simulated `admin()` read returned `GBHVPV4S3JRAOPQYODNULNPBW57REIJ2SDNAHYHN6SXGOKJCS3XLPVZD`.

## Remaining boundaries

Business-flow network smoke tests and browser-wallet integration have not been performed.
Local tests do not establish real-user outcomes. No mainnet deployment is authorized.
Pilot deployment still requires the documented real-user agreement. Testnet resets
and storage expiry can make this address or its state unavailable later.
