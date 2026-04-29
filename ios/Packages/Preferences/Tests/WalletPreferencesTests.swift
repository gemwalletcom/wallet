// Copyright (c). Gem Wallet. All rights reserved.

@testable import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import Testing

struct WalletPreferencesTests {
    private let preferences: WalletPreferences = .mock()
    private let asset: Asset = .mock()

    @Test
    func defaultPreferences() {
        #expect(preferences.assetsTimestamp == 0)
        #expect(preferences.transactionsTimestamp == 0)
        #expect(!preferences.completeInitialLoadAssets)
        #expect(!preferences.completeInitialLoadTransactions)
        #expect(!preferences.completeInitialLoadNFTs)
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

        preferences.completeInitialLoadTransactions = true
        #expect(preferences.completeInitialLoadTransactions)

        preferences.completeInitialLoadNFTs = true
        #expect(preferences.completeInitialLoadNFTs)

        preferences.completeInitialAddressStatus = true
        #expect(preferences.completeInitialAddressStatus)

        preferences.setTransactionsForAssetTimestamp(assetId: asset.id.identifier, value: 10)
        #expect(preferences.transactionsForAssetTimestamp(assetId: asset.id.identifier) == 10)
    }

    @Test
    func completeInitialSynchronization() {
        preferences.completeInitialSynchronization()

        #expect(preferences.completeInitialLoadAssets)
        #expect(preferences.completeInitialLoadTransactions)
        #expect(preferences.completeInitialLoadNFTs)
        #expect(preferences.completeInitialAddressStatus)
    }

    @Test
    func testClear() {
        preferences.assetsTimestamp = 123
        preferences.transactionsTimestamp = 456
        preferences.completeInitialLoadAssets = true
        preferences.completeInitialLoadTransactions = true
        preferences.completeInitialLoadNFTs = true
        preferences.completeInitialAddressStatus = true
        preferences.setTransactionsForAssetTimestamp(assetId: asset.id.identifier, value: 10)

        #expect(preferences.assetsTimestamp == 123)
        #expect(preferences.transactionsTimestamp == 456)
        #expect(preferences.completeInitialLoadAssets)
        #expect(preferences.completeInitialLoadTransactions)
        #expect(preferences.completeInitialLoadNFTs)
        #expect(preferences.completeInitialAddressStatus)
        #expect(preferences.transactionsForAssetTimestamp(assetId: asset.id.identifier) == 10)

        preferences.clear()

        #expect(preferences.assetsTimestamp == 0)
        #expect(preferences.transactionsTimestamp == 0)
        #expect(!preferences.completeInitialLoadAssets)
        #expect(!preferences.completeInitialLoadTransactions)
        #expect(!preferences.completeInitialLoadNFTs)
        #expect(!preferences.completeInitialAddressStatus)
        #expect(preferences.transactionsForAssetTimestamp(assetId: asset.id.identifier) == 0)
    }
}
