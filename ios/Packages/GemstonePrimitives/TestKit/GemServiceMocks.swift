// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitives
import Primitives

public actor GemDeviceServiceMock: GemDeviceServiceProtocol {
    private let delay: Duration?
    private let needsSyncResult: Bool
    private let syncError: Error?

    public private(set) var needsSyncCalls = 0
    public private(set) var syncCalls = 0
    public private(set) var syncedDeviceIds: [String] = []

    public init(
        delay: Duration? = nil,
        needsSync: Bool = true,
        syncError: Error? = nil,
    ) {
        self.delay = delay
        needsSyncResult = needsSync
        self.syncError = syncError
    }

    public func needsSync(device _: Gemstone.Device) async throws -> Bool {
        needsSyncCalls += 1
        return needsSyncResult
    }

    public func sync(device: Gemstone.Device) async throws -> Gemstone.Device {
        syncCalls += 1
        syncedDeviceIds.append(try Primitives.Device(device).id)
        try await sleepIfNeeded()
        if let syncError {
            throw syncError
        }
        return device
    }

    private func sleepIfNeeded() async throws {
        if let delay {
            try await Task.sleep(for: delay)
        }
    }
}

public final class GemPreferencesStoreMock: GemPreferencesStore, @unchecked Sendable {
    private let lock = NSLock()
    private var values: [String: String] = [:]

    public init() {}

    public func get(key: String) throws -> String? {
        lock.withLock { values[key] }
    }

    public func set(key: String, value: String) throws {
        lock.withLock { values[key] = value }
    }

    public func remove(key: String) throws {
        lock.withLock { values[key] = nil }
    }
}

public final class GemPreferencesServiceMock: GemPreferencesServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var priceAlertsEnabled: Bool
    private var skippedAppVersion: String?

    public init(priceAlertsEnabled: Bool = false) {
        self.priceAlertsEnabled = priceAlertsEnabled
    }

    public func getSkippedAppVersion() throws -> String? {
        lock.withLock { skippedAppVersion }
    }

    public func setSkippedAppVersion(version: String) throws {
        lock.withLock { skippedAppVersion = version }
    }

    public func isPriceAlertsEnabled() throws -> Bool {
        lock.withLock { priceAlertsEnabled }
    }

    public func setPriceAlertsEnabled(enabled: Bool) throws {
        lock.withLock { priceAlertsEnabled = enabled }
    }
}

public final class GemPriceAlertServiceMock: GemPriceAlertServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var enabled: Bool

    public init(enabled: Bool = false) {
        self.enabled = enabled
    }

    public func isEnabled() throws -> Bool {
        lock.withLock { enabled }
    }

    public func setEnabled(enabled: Bool) async throws {
        lock.withLock { self.enabled = enabled }
    }

    public func sync(assetId _: String?) async throws {}

    public func enablePriceAlert(alert _: Gemstone.PriceAlert) async throws {
        lock.withLock { enabled = true }
    }

    public func addPriceAlerts(alerts _: [Gemstone.PriceAlert]) async throws {}

    public func deletePriceAlerts(alerts _: [Gemstone.PriceAlert]) async throws {}

    public func priceAlertId(alert: Gemstone.PriceAlert) -> String {
        (try? Primitives.PriceAlert(alert).id) ?? ""
    }
}

public final class GemTransactionsServiceMock: GemTransactionsServiceProtocol, @unchecked Sendable {
    public typealias Sync = @Sendable (String, Gemstone.AssetId?) async throws -> Void

    private let lock = NSLock()
    private var onSync: Sync

    public init(onSync: @escaping Sync = { _, _ in }) {
        self.onSync = onSync
    }

    public func setOnSync(_ onSync: @escaping Sync) {
        lock.withLock { self.onSync = onSync }
    }

    public func sync(walletId: String, assetId: Gemstone.AssetId?) async throws {
        try await lock.withLock { onSync }(walletId, assetId)
    }

}

public final class GemContactServiceMock: GemContactServiceProtocol, @unchecked Sendable {
    public init() {}

    public func addContact(contact _: Gemstone.Contact, addresses _: [Gemstone.ContactAddress]) async throws {}

    public func updateContact(contact _: Gemstone.Contact, addresses _: [Gemstone.ContactAddress]) async throws {}

    public func deleteContact(contact _: Gemstone.Contact) async throws {}

    public func saveAvatar(image _: Data) throws -> String {
        ""
    }

    public func removeAvatar(fileName _: String) throws {}
}

public final class GemStreamServiceMock: GemStreamServiceProtocol, @unchecked Sendable {
    public init() {}

    public func handle(event _: Gemstone.StreamEvent, currency _: Gemstone.Currency) async throws {}
}

public final class GemFiatServiceMock: GemFiatServiceProtocol, @unchecked Sendable {
    private let quotes: [Primitives.FiatQuote]

    public init(quotes: [Primitives.FiatQuote] = []) {
        self.quotes = quotes
    }

    public func syncTransactions(walletId _: String) async throws {}

    public func getQuotes(walletId _: String, quoteType _: Gemstone.FiatQuoteType, assetId _: String, amount _: Double, currency _: String) async throws -> [Gemstone.FiatQuote] {
        try quotes.map { try $0.json() }
    }

    public func getQuoteUrl(walletId _: String, quoteId _: String) async throws -> Gemstone.FiatQuoteUrl {
        throw AnyError("not stubbed")
    }

}

public final class GemNameServiceMock: GemNameServiceProtocol, @unchecked Sendable {
    private let addressNames: [Primitives.AddressName]
    private let nameRecord: Primitives.NameRecord?
    private let error: Error?
    public private(set) var resolvedNames: [String] = []

    public init(addressNames: [Primitives.AddressName] = [], nameRecord: Primitives.NameRecord? = nil, error: Error? = nil) {
        self.addressNames = addressNames
        self.nameRecord = nameRecord
        self.error = error
    }

    public func resolve(name: String, chain _: String) async throws -> Gemstone.NameRecord? {
        resolvedNames.append(name)
        if let error { throw error }
        return try nameRecord?.json()
    }

    public func canResolveName(name: String) -> Bool {
        name.split(separator: ".").count >= 2
    }

    public func getAddressNames(requests: [Gemstone.ChainAddress]) async throws -> [Gemstone.AddressName] {
        if error != nil { return [] }
        let requested = try requests.map { try Primitives.ChainAddress($0) }
        return try addressNames
            .filter { name in requested.contains { $0.chain == name.chain && $0.address == name.address } }
            .map { try $0.json() }
    }
}

public final class GemPortfolioServiceMock: GemPortfolioServiceProtocol, @unchecked Sendable {
    private let allTimeHigh: ChartValuePercentage?
    private let allTimeLow: ChartValuePercentage?

    public init(allTimeHigh: ChartValuePercentage? = nil, allTimeLow: ChartValuePercentage? = nil) {
        self.allTimeHigh = allTimeHigh
        self.allTimeLow = allTimeLow
    }

    public func getAssets(period _: Gemstone.ChartPeriod, request _: Gemstone.PortfolioAssetsRequest) async throws -> Gemstone.PortfolioAssets {
        try Primitives.PortfolioAssets(totalValue: 0, values: [], allTimeHigh: allTimeHigh, allTimeLow: allTimeLow, allocation: []).json()
    }

    public func getWalletAssets(walletId _: Gemstone.WalletId, period: Gemstone.ChartPeriod) async throws -> Gemstone.PortfolioAssets {
        try await getAssets(period: period, request: Primitives.PortfolioAssetsRequest(assets: []).json())
    }
}

public final class GemStakeServiceMock: GemStakeServiceProtocol, @unchecked Sendable {
    private let earnData: String

    public init(earnData: String = "{}") {
        self.earnData = earnData
    }

    public func sync(walletId _: String, chain _: Gemstone.Chain, address _: String) async throws {}

    public func syncEarn(walletId _: String, assetId _: Gemstone.AssetId, address _: String) async throws {}

    public func getEarnData(assetId _: Gemstone.AssetId, address _: String, value _: String, earnType _: Gemstone.EarnType) async throws -> Gemstone.ContractCallData {
        earnData
    }
}

public final class GemTransactionStateServiceMock: GemTransactionStateServiceProtocol, @unchecked Sendable {
    private let store: (any GemTransactionStateStore)?
    private let update: @Sendable (String, Gemstone.Transaction) async throws -> GemTransactionStateResult?

    public init(store: (any GemTransactionStateStore)? = nil, update: @escaping @Sendable (String, Gemstone.Transaction) async throws -> GemTransactionStateResult?) {
        self.store = store
        self.update = update
    }

    public func update(walletId: String, transaction: Gemstone.Transaction) async throws -> GemTransactionStateResult? {
        try await update(walletId, transaction)
    }

    public func pendingTransactions() async throws -> [GemPendingTransaction] {
        try await store?.getPendingTransactions() ?? []
    }

    public func getTransaction(walletId: Gemstone.WalletId, transactionId: Gemstone.TransactionId) async throws -> GemPendingTransaction? {
        try await store?.getTransaction(walletId: walletId, transactionId: transactionId)
    }

    public func addTransactions(walletId: Gemstone.WalletId, transactions: [Gemstone.Transaction]) async throws {
        try await store?.addTransactions(walletId: walletId, transactions: transactions)
    }
}

public final class GemBalanceServiceMock: GemBalanceServiceProtocol, @unchecked Sendable {
    private let onUpdate: @Sendable (String, [Gemstone.AssetId]) async -> Void

    public init(onUpdate: @escaping @Sendable (String, [Gemstone.AssetId]) async -> Void = { _, _ in }) {
        self.onUpdate = onUpdate
    }

    public func update(walletId: String, assetIds: [Gemstone.AssetId]) async throws {
        await onUpdate(walletId, assetIds)
    }

    public func enableAssets(walletId _: String, assetIds _: [Gemstone.AssetId], enabled _: Bool, currency _: Gemstone.Currency) async throws {}

    public func pinAsset(walletId _: String, assetId _: Gemstone.AssetId, pinned _: Bool, currency _: Gemstone.Currency) async throws {}
}

public final class GemPerpetualServiceMock: GemPerpetualServiceProtocol, @unchecked Sendable {
    public private(set) var syncMarketsCount = 0
    public private(set) var clearMarketsCount = 0
    private var updatedAt: Int64?

    public init(marketsUpdatedAt: Int64? = nil) {
        updatedAt = marketsUpdatedAt
    }

    public func marketsUpdatedAt() throws -> Int64? {
        updatedAt
    }

    public func syncMarkets(chain _: Gemstone.Chain) async throws {
        syncMarketsCount += 1
        updatedAt = Int64(Date().timeIntervalSince1970)
    }

    public func clearMarkets() async throws {
        clearMarketsCount += 1
        updatedAt = nil
    }

    public func syncPositions(walletId _: String, chain _: Gemstone.Chain, address _: String) async throws {}

    public func setPinned(perpetualId _: String, pinned _: Bool) async throws {}

    public func getPositions(walletId _: String, chain _: Gemstone.Chain) async throws -> [Gemstone.PerpetualPosition] {
        []
    }

    public func updatePositions(walletId _: String, positions _: [Gemstone.PerpetualPosition], deleteIds _: [String]) async throws {}

    public func updateBalance(walletId _: String, balance _: Gemstone.PerpetualBalance) async throws {}

    public func updateMarket(market _: Gemstone.PerpetualMarketData) async throws {}

    public func updatePrices(prices _: [String: Double]) async throws {}
}

public final class GemAssetDiscoveryServiceMock: GemAssetDiscoveryServiceProtocol, @unchecked Sendable {
    public init() {}

    public func discover(walletId _: String, currency _: Gemstone.Currency) async throws -> [Gemstone.AssetId] {
        []
    }
}

public final class GemExplorerServiceMock: GemExplorerServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var names: [Gemstone.Chain: String] = [:]

    public init() {}

    public func getExplorers(chain _: Gemstone.Chain) -> [String] {
        ["MockExplorer"]
    }

    public func getExplorerName(chain: Gemstone.Chain) -> String {
        lock.withLock { names[chain] ?? "MockExplorer" }
    }

    public func setExplorerName(chain: Gemstone.Chain, name: String) throws {
        lock.withLock { names[chain] = name }
    }

    public func getTransactionUrl(chain: Gemstone.Chain, hash: String) -> GemBlockExplorerLink {
        link("https://mock.explorer/\(chain)/tx/\(hash)")
    }

    public func getTransactionLink(chain: Gemstone.Chain, hash: String, provider _: String?, recipient _: String?, memo _: String?) -> GemBlockExplorerLink {
        getTransactionUrl(chain: chain, hash: hash)
    }

    public func getAddressUrl(chain: Gemstone.Chain, address: String) -> GemBlockExplorerLink {
        link("https://mock.explorer/\(chain)/address/\(address)")
    }

    public func getTokenUrl(chain: Gemstone.Chain, address: String) -> GemBlockExplorerLink? {
        link("https://mock.explorer/\(chain)/token/\(address)")
    }

    public func getNftUrl(chain: Gemstone.Chain, contractAddress: String, tokenId: String) -> GemBlockExplorerLink? {
        link("https://mock.explorer/\(chain)/nft/\(contractAddress)/\(tokenId)")
    }

    public func getValidatorUrl(chain: Gemstone.Chain, address: String) -> GemBlockExplorerLink? {
        link("https://mock.explorer/\(chain)/validator/\(address)")
    }

    private func link(_ url: String) -> GemBlockExplorerLink {
        GemBlockExplorerLink(name: "MockExplorer", link: url)
    }
}

public extension GemExplorerService {
    static func mock() -> GemExplorerService {
        GemExplorerService(preferences: GemPreferencesService(store: GemPreferencesStoreMock()))
    }
}

public final class GemBannerServiceMock: GemBannerServiceProtocol, @unchecked Sendable {
    public private(set) var closedKeys: [GemBannerKey] = []
    public private(set) var handledActions: [GemBannerAction] = []

    public init() {}

    public func activeEvents(walletId _: Gemstone.WalletId?, assetId _: Gemstone.AssetId?, context _: GemBannerContext) async throws -> [Gemstone.BannerEvent] {
        []
    }

    public func close(key: GemBannerKey) async throws {
        closedKeys.append(key)
    }

    public func closesOnAction(event _: Gemstone.BannerEvent) -> Bool {
        false
    }

    public func handleAction(key _: GemBannerKey, action: GemBannerAction) async throws {
        handledActions.append(action)
    }

    public func setup() async throws {}

    public func setupWallet(wallet _: Gemstone.Wallet) async throws {}
}

public final class GemSearchServiceMock: GemSearchServiceProtocol, @unchecked Sendable {
    private let assets: [Primitives.AssetBasic]

    public init(assets: [Primitives.AssetBasic] = []) {
        self.assets = assets
    }

    public func search(wallet _: Gemstone.Wallet, query _: String, scope _: GemSearchScope, currency _: Gemstone.Currency) async throws -> Bool {
        !assets.isEmpty
    }

    public func searchAssets(wallet _: Gemstone.Wallet, query _: String, currency _: Gemstone.Currency) async throws -> [Gemstone.AssetBasic] {
        try assets.map { try $0.json() }
    }
}

public final class GemStreamSubscriptionServiceMock: GemStreamSubscriptionServiceProtocol, @unchecked Sendable {
    public init() {}

    public func setupAssets(walletId _: Gemstone.WalletId) async throws {}

    public func resubscribe() async throws {}

    public func addPrices(assetIds _: [Gemstone.AssetId]) async throws {}

    public func reset() {}
}

public final class GemAvatarServiceMock: GemAvatarServiceProtocol, @unchecked Sendable {
    public init() {}

    public func setImage(walletId _: Gemstone.WalletId, image _: Data) async throws {}

    public func setImageUrl(walletId _: Gemstone.WalletId, url _: String) async throws {}

    public func removeImage(walletId _: Gemstone.WalletId) async throws {}
}
