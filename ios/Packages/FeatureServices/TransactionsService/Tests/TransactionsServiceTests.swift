// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Preferences
import Primitives
import PrimitivesTestKit
import Testing
import TransactionsService
import TransactionsServiceTestKit

struct TransactionsServiceTests {
    @Test
    func updateForAssetAdvancesTimestampOnEmptyResponse() async throws {
        let walletId = WalletId.mock(address: UUID().uuidString)
        let assetId = AssetId.mockEthereum()
        let preferences = WalletPreferences(walletId: walletId)

        try await TransactionsService.mock().updateForAsset(walletId: walletId, assetId: assetId)

        #expect(preferences.transactionsForAssetTimestamp(assetId: assetId.identifier) > 0)
    }
}
