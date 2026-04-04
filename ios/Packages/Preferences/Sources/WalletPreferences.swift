// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public final class WalletPreferences: @unchecked Sendable {
    private enum Keys {
        static let assetsTimestamp = "assets_timestamp"
        static let transactionsForAsset = "transactions_for_asset_v1"
        static let transactionsTimestamp = "transactions_timestamp_v1"
        static let notificationsTimestamp = "notifications_timestamp"
        static let completeInitialLoadAssets = "complete_initial_load_assets"
        static let completeInitialLoadTransactions = "complete_initial_load_transactions"
        static let completeInitialLoadNFTs = "complete_initial_load_nfts"
        static let completeInitialAddressStatus = "complete_initial_address_status"
    }

    private let defaults: UserDefaults
    private let suiteName: String

    public init(walletId: WalletId) {
        suiteName = Self.suiteName(walletId: walletId.id)
        defaults = UserDefaults(suiteName: suiteName)!
    }

    public var completeInitialLoadAssets: Bool {
        set { defaults.setValue(newValue, forKey: Keys.completeInitialLoadAssets) }
        get { defaults.bool(forKey: Keys.completeInitialLoadAssets) }
    }

    public var completeInitialLoadTransactions: Bool {
        set { defaults.setValue(newValue, forKey: Keys.completeInitialLoadTransactions) }
        get { defaults.bool(forKey: Keys.completeInitialLoadTransactions) }
    }

    public var completeInitialLoadNFTs: Bool {
        set { defaults.setValue(newValue, forKey: Keys.completeInitialLoadNFTs) }
        get { defaults.bool(forKey: Keys.completeInitialLoadNFTs) }
    }

    public var transactionsTimestamp: Int {
        set { defaults.setValue(newValue, forKey: Keys.transactionsTimestamp) }
        get { defaults.integer(forKey: Keys.transactionsTimestamp) }
    }

    public var notificationsTimestamp: Int {
        set { defaults.setValue(newValue, forKey: Keys.notificationsTimestamp) }
        get { defaults.integer(forKey: Keys.notificationsTimestamp) }
    }

    public var assetsTimestamp: Int {
        set { defaults.setValue(newValue, forKey: Keys.assetsTimestamp) }
        get { defaults.integer(forKey: Keys.assetsTimestamp) }
    }

    public var completeInitialAddressStatus: Bool {
        set { defaults.setValue(newValue, forKey: Keys.completeInitialAddressStatus) }
        get { defaults.bool(forKey: Keys.completeInitialAddressStatus) }
    }

    public func completeInitialSynchronization() {
        completeInitialAddressStatus = true
        completeInitialLoadAssets = true
        completeInitialLoadTransactions = true
        completeInitialLoadNFTs = true
    }

    // transactions
    public func setTransactionsForAssetTimestamp(assetId: String, value: Int) {
        defaults.setValue(value, forKey: String(format: "%@_%@", Keys.transactionsForAsset, assetId))
    }

    public func transactionsForAssetTimestamp(assetId: String) -> Int {
        defaults.integer(forKey: String(format: "%@_%@", Keys.transactionsForAsset, assetId))
    }

    public func clear() {
        UserDefaults.standard.removePersistentDomain(forName: suiteName)
    }

    private static func suiteName(walletId: String) -> String {
        "wallet_preferences_\(walletId)_v2"
    }
}
