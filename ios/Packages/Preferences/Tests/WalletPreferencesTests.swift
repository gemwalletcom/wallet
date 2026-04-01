// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import Testing

@testable import Preferences

struct WalletPreferencesTests {
    private let preferences: WalletPreferences = .mock()
    private let asset: Asset = .mock()

    @Test
    func defaultPreferences() {
        #expect(preferences.assetsTimestamp == 0)
        #expect(preferences.transactionsTimestamp == 0)
        #expect(!preferences.completeInitialLoadAssets)
        #expect(!preferences.completeInitialAddressStatus)
        #expect(preferences.transactionsForAssetTimestamp(assetId: asset.id.identifier) == 0)
    }

    @Test
    func updatePreferences() {
        preferences.assetsTimestamp = 123
        #expect(preferences.assetsTimestamp == 123)

        preferences.transactionsTimestamp = 456
        #expect(preferences.transactionsTimestamp == 456)

        preferences.completeInitialLoadAssets = true
        #expect(preferences.completeInitialLoadAssets)

        preferences.completeInitialAddressStatus = true
        #expect(preferences.completeInitialAddressStatus)

        preferences.setTransactionsForAssetTimestamp(assetId: asset.id.identifier, value: 10)
        #expect(preferences.transactionsForAssetTimestamp(assetId: asset.id.identifier) == 10)
    }

    @Test
    func testClear() {
        preferences.assetsTimestamp = 123
        preferences.transactionsTimestamp = 456
        preferences.completeInitialLoadAssets = true
        preferences.completeInitialAddressStatus = true
        preferences.setTransactionsForAssetTimestamp(assetId: asset.id.identifier, value: 10)

        #expect(preferences.assetsTimestamp == 123)
        #expect(preferences.transactionsTimestamp == 456)
        #expect(preferences.completeInitialLoadAssets)
        #expect(preferences.completeInitialAddressStatus)
        #expect(preferences.transactionsForAssetTimestamp(assetId: asset.id.identifier) == 10)

        preferences.clear()

        #expect(preferences.assetsTimestamp == 0)
        #expect(preferences.transactionsTimestamp == 0)
        #expect(!preferences.completeInitialLoadAssets)
        #expect(!preferences.completeInitialAddressStatus)
        #expect(preferences.transactionsForAssetTimestamp(assetId: asset.id.identifier) == 0)
    }
}
