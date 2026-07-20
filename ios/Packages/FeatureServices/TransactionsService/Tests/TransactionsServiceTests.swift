// Copyright (c). Gem Wallet. All rights reserved.

import Preferences
import Primitives
import PrimitivesTestKit
import Testing
import TransactionsService
import TransactionsServiceTestKit

struct TransactionsServiceTests {
    @Test
    func updateForAssetAdvancesTimestampOnEmptyResponse() async throws {
        let walletId = WalletId.mock()
        let assetId = AssetId.mockEthereum()
        let preferences = WalletPreferences(walletId: walletId)
        preferences.clear()

        try await TransactionsService.mock().updateForAsset(walletId: walletId, assetId: assetId)

        #expect(preferences.transactionsForAssetTimestamp(assetId: assetId.identifier) > 0)
    }
}
