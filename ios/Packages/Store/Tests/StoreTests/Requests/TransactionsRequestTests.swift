// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Store
import Testing

struct TransactionsRequestTests {
    @Test
    func assetScene() {
        let walletId = WalletId.multicoin(address: "wallet")
        let assetId = AssetId(chain: .ethereum)

        #expect(
            TransactionsRequest.assetScene(walletId: walletId, assetId: assetId, limit: 250) ==
                TransactionsRequest(
                    walletId: walletId,
                    type: .asset(assetId: assetId),
                    limit: 250,
                ),
        )
    }

    @Test
    func perpetualScene() {
        let walletId = WalletId.multicoin(address: "wallet")
        let assetId = Asset.mockHypercoreUSDC().id

        #expect(
            TransactionsRequest.perpetualScene(
                walletId: walletId,
                assetId: assetId,
                limit: 250,
            ) ==
                TransactionsRequest(
                    walletId: walletId,
                    type: .asset(assetId: assetId),
                    filters: [.types([
                        TransactionType.perpetualOpenPosition.rawValue,
                        TransactionType.perpetualClosePosition.rawValue,
                    ])],
                    limit: 250,
                ),
        )
    }
}
