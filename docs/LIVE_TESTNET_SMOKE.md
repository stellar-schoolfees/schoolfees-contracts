# SchoolFees live testnet smoke test

Completed 2026-10-08T21:48:41.973Z. This exercised the deployed contract through the Stellar CLI with dedicated synthetic testnet accounts. It did not use the browser app, a browser wallet, a real school, personal data or mainnet.

Contract: `CBRSL5YIUN3ANQVEJB4YWQAXBN6KMZR55OWBDZS7ZLHCRKLDOOYWAXLR`. Fee ID: `1`.
Token: testnet native SAC `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`.

| Action | Transaction | Ledger |
|---|---|---|
| create fee | [SUCCESS](https://stellar.expert/explorer/testnet/tx/fa89ac557fb183a887c60e53164c74e609051f9b0a2fb5aed1aeb14af2ae41f6) | 5094434 |
| partial payment | [SUCCESS](https://stellar.expert/explorer/testnet/tx/5e0125689d3428d219e185ca421518dbcc2a506090028362ae52dfaee7ef8968) | 5094477 |
| refund | [SUCCESS](https://stellar.expert/explorer/testnet/tx/d92a9562b18e41d655fe0ea08056ae69839f0fcc5817117f0d4e3a9552d0f798) | 5094479 |
| remaining payment | [SUCCESS](https://stellar.expert/explorer/testnet/tx/2d75cf0100d846d918c8d377513184e12b86d090768d37dc3221a61dba20a8f1) | 5094480 |
| close paid fee | [SUCCESS](https://stellar.expert/explorer/testnet/tx/219252fcf50c435c97bf31ae65bbcc411dd5d11fe6829a0ac96512081c21e331) | 5094482 |

Verified recorded totals: gross payments 11,000,000 atomic units; refunds 1,000,000; net payment 10,000,000; final status Closed. The school received exactly the net amount and the payer lost exactly that amount after accounting for the actual on-chain transaction fees. Balances and fee charges are in the JSON evidence.

RPC simulations rejected overpayment (31), closing a partially paid fee (11), an excessive refund (33) and payment after closure (10). These negative simulations sent no transaction.

An initial test-harness parser mistook the opaque reference for the transaction hash. The original creation was subsequently verified by its actual explorer transaction hash and never resubmitted. Fee decoding was corrected offline for the installed SDK's property-style accessor. Neither correction changed contract or app code.

[Machine-readable evidence](schoolfees-live-smoke.json) contains the actual records, RPC transaction results and assertions. Browser-wallet integration, different token configurations, manual accessibility and real-user testing remain unverified. This smoke test is not an audit or pilot.
