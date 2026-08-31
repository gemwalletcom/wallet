// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public actor GemDeviceServiceMock: GemDeviceServiceProtocol {
    private let syncError: Error?
    public private(set) var synchronizeCalls = 0
    public private(set) var synchronizeIfNeededCalls = 0
    public private(set) var pushEnabledValues: [Bool] = []

    public init(syncError: Error? = nil) {
        self.syncError = syncError
    }

    public func synchronize() async throws -> Gemstone.Device {
        synchronizeCalls += 1
        if let syncError {
            throw syncError
        }
        return try Primitives.Device.mock().json()
    }

    public func isRegistered() async throws -> Bool { true }

    public func setPushEnabled(enabled: Bool) async throws {
        pushEnabledValues.append(enabled)
        if let syncError {
            throw syncError
        }
    }

    public func synchronizeIfNeeded() async throws {
        synchronizeIfNeededCalls += 1
        if let syncError {
            throw syncError
        }
    }
}

public final class GemSecureStoreMock: GemSecureStore, @unchecked Sendable {
    private var values: [String: String] = [:]

    public init() {}

    public func get(key: String) throws -> String? { values[key] }

    public func set(key: String, value: String) throws { values[key] = value }

    public func remove(key: String) throws { values[key] = nil }
}

public final class GemPreferencesStoreMock: GemPreferencesStore, @unchecked Sendable {
    private let lock = NSLock()
    private var values: [String: String] = [:]

    public init() {}

    public func get(key: String) -> String? {
        lock.withLock { values[key] }
    }

    public func set(key: String, value: String) throws {
        lock.withLock { values[key] = value }
    }

    public func remove(key: String) throws {
        lock.withLock { values[key] = nil }
    }

    public func clear() throws {
        lock.withLock { values.removeAll() }
    }
}

public final class GemPreferencesServiceMock: GemPreferencesServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var priceAlertsEnabled: Bool
    private var skippedAppVersion: String?

    public init(priceAlertsEnabled: Bool = false) {
        self.priceAlertsEnabled = priceAlertsEnabled
    }

    public func isPriceAlertsEnabled() -> Bool {
        lock.withLock { priceAlertsEnabled }
    }

    public func setPriceAlertsEnabled(enabled: Bool) throws {
        lock.withLock { priceAlertsEnabled = enabled }
    }

    public func getCurrency() -> Gemstone.Currency { (Primitives.Currency.usd.json()) ?? "\"USD\"" }

    public func setCurrency(currency _: Gemstone.Currency) throws {}

    public func setupCurrency(localeCurrency _: String?) throws -> Gemstone.Currency { Primitives.Currency.usd.json() }

    public func getChartPeriod() -> Gemstone.ChartPeriod { (Primitives.ChartPeriod.day.json()) ?? "\"day\"" }

    public func setChartPeriod(period _: Gemstone.ChartPeriod) throws {}

    public func getPerpetualChartPeriod() -> Gemstone.ChartPeriod { (Primitives.ChartPeriod.day.json()) ?? "\"day\"" }

    public func setPerpetualChartPeriod(period _: Gemstone.ChartPeriod) throws {}

    public func isPushNotificationsEnabled() -> Bool { false }

    public func setPushNotificationsEnabled(enabled _: Bool) throws {}

    private var perpetualEnabled = false
    private var hideBalanceEnabled = false
    private var developerEnabled = false
    private var acceptTermsCompleted = false
    private var appearance: Gemstone.Appearance = (Primitives.Appearance.system.json()) ?? "\"system\""

    public func isPerpetualEnabled() -> Bool { perpetualEnabled }

    public func setPerpetualEnabled(enabled: Bool) throws { perpetualEnabled = enabled }

    public func showPerpetuals(wallet: Gemstone.Wallet) -> Bool {
        perpetualEnabled && ((try? Primitives.Wallet(wallet).hasPerpetualsSupport) ?? false)
    }

    public func isHideBalanceEnabled() -> Bool { hideBalanceEnabled }

    public func setHideBalanceEnabled(enabled: Bool) throws { hideBalanceEnabled = enabled }

    public func isDeveloperEnabled() -> Bool { developerEnabled }

    public func setDeveloperEnabled(enabled: Bool) throws { developerEnabled = enabled }

    public func isAcceptTermsCompleted() -> Bool { acceptTermsCompleted }

    public func setAcceptTermsCompleted() throws { acceptTermsCompleted = true }

    public func getAppearance() -> Gemstone.Appearance { appearance }

    public func setAppearance(appearance: Gemstone.Appearance) throws { self.appearance = appearance }

    public func getSwapSlippageBps() -> UInt32? { nil }

    public func setSwapSlippageBps(bps _: UInt32?) throws {}

    public func getPerpetualLeverage() -> UInt8 { 5 }

    public func setPerpetualLeverage(leverage _: UInt8) throws {}

    public func getPerpetualTakeProfitPercent() -> UInt8 { 0 }

    public func setPerpetualTakeProfitPercent(percent _: UInt8) throws {}

    public func getPerpetualStopLossPercent() -> UInt8 { 0 }

    public func setPerpetualStopLossPercent(percent _: UInt8) throws {}

    public func getLaunchesCount() -> UInt32 { 0 }

    public func incrementLaunchesCount() throws -> UInt32 { 1 }

    public func shouldRequestReview() -> Bool { false }

    public func setRateApplicationShown() throws {}

    public func shouldAskNotifications() -> Bool { false }

    public func setNotificationsAsked() throws {}

    public func defaultCurrency(localeCurrency _: String?) -> Gemstone.Currency {
        "\"USD\""
    }

    public func clear() throws {}
}

public final class GemPriceAlertServiceMock: GemPriceAlertServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var enabled: Bool

    public init(enabled: Bool = false) {
        self.enabled = enabled
    }

    public func isEnabled() -> Bool {
        lock.withLock { enabled }
    }

    public func setEnabled(enabled: Bool) async throws {
        lock.withLock { self.enabled = enabled }
    }

    public func sync(assetId _: String?) async throws {}

    public func enablePriceAlert(alert _: Gemstone.PriceAlert) async throws {
        lock.withLock { enabled = true }
    }

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
    private let service = GemContactService(
        store: GemContactStoreMock(),
        addressStore: GemAddressStoreMock(),
        files: GemFileStoreMock(),
    )

    public init() {}

    public func saveContact(input: GemContactInput) async throws -> Gemstone.Contact {
        try await service.saveContact(input: input)
    }

    public func updateContact(contact: Gemstone.Contact, addresses: [Gemstone.ContactAddress]) async throws {
        try await service.updateContact(contact: contact, addresses: addresses)
    }

    public func deleteContact(contact: Gemstone.Contact) async throws {
        try await service.deleteContact(contact: contact)
    }

    public func addAddress(addresses: [Gemstone.ContactAddress], input: GemContactAddressInput) -> [Gemstone.ContactAddress] {
        service.addAddress(addresses: addresses, input: input)
    }

    public func defaultChain() -> Gemstone.Chain {
        service.defaultChain()
    }
}

public final class GemContactStoreMock: GemContactStore, @unchecked Sendable {
    public init() {}

    public func getAddresses(contactId _: String) async throws -> [Gemstone.ContactAddress] { [] }

    public func saveContact(contact _: Gemstone.Contact, addresses _: [Gemstone.ContactAddress]) async throws {}

    public func updateContact(contact _: Gemstone.Contact, addresses _: [Gemstone.ContactAddress], deleteAddressIds _: [String]) async throws {}

    public func deleteContact(contactId _: String) async throws {}
}

public final class GemAddressStoreMock: GemAddressStore, @unchecked Sendable {
    public init() {}

    public func getAddressName(chain _: Gemstone.Chain, address _: String) throws -> Gemstone.AddressName? { nil }

    public func saveAddressNames(names _: [Gemstone.AddressName]) async throws {}

    public func deleteAddressNames(names _: [Gemstone.AddressName]) async throws {}
}

public final class GemFileStoreMock: GemFileStore, @unchecked Sendable {
    public init() {}

    public func saveFile(data _: Data, extension _: String) throws -> String { "" }

    public func saveNamedFile(data _: Data, fileName: String) throws -> String { fileName }

    public func exists(fileName _: String) -> Bool { false }

    public func path(fileName: String) -> String { fileName }

    public func remove(fileName _: String) throws {}
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

    public func quoteDebounceMilliseconds() -> UInt64 { 250 }

    public func quoteRefreshIntervalMilliseconds() -> UInt64 { 300_000 }

    public func syncTransactions(walletId _: String) async throws {}

    public func getQuotes(walletId _: String, quoteType _: Gemstone.FiatQuoteType, assetId _: String, amount _: Double, currency _: String) async throws -> [Gemstone.FiatQuote] {
        quotes.map { $0.json() }
    }

    public func getQuoteUrl(walletId _: String, quoteId _: String) async throws -> Gemstone.FiatQuoteUrl {
        throw AnyError("not stubbed")
    }

}

public final class GemNameServiceMock: GemNameServiceProtocol, @unchecked Sendable {
    private let recipientService = GemRecipientService()
    private let addressNames: [Primitives.AddressName]
    private let nameRecord: Primitives.NameRecord?
    private let error: Error?
    public private(set) var requestedNames: [String] = []

    public init(addressNames: [Primitives.AddressName] = [], nameRecord: Primitives.NameRecord? = nil, error: Error? = nil) {
        self.addressNames = addressNames
        self.nameRecord = nameRecord
        self.error = error
    }

    public func getNameRecord(name: String, chain _: String) async throws -> Gemstone.NameRecord? {
        requestedNames.append(name)
        if let error { throw error }
        return try nameRecord?.json()
    }

    public func isNameSupported(name: String) -> Bool {
        name.split(separator: ".").count >= 2
    }

    public func addressName(chain: String, address: String) throws -> Gemstone.AddressName? {
        try addressNames.first { $0.chain.rawValue == chain && $0.address == address }?.json()
    }

    public func recipients() -> GemRecipientService {
        recipientService
    }

    public func getAddressNames(requests: [Gemstone.ChainAddress]) async throws -> [Gemstone.AddressName] {
        if error != nil { return [] }
        let requested = try requests.map { try Primitives.ChainAddress($0) }
        return try addressNames
            .filter { name in requested.contains { $0.chain == name.chain && $0.address == name.address } }
            .map { $0.json() }
    }
}

public final class GemPortfolioServiceMock: GemPortfolioServiceProtocol, @unchecked Sendable {
    private let allTimeHigh: Primitives.ChartValuePercentage?
    private let allTimeLow: Primitives.ChartValuePercentage?

    public init(allTimeHigh: Primitives.ChartValuePercentage? = nil, allTimeLow: Primitives.ChartValuePercentage? = nil) {
        self.allTimeHigh = allTimeHigh
        self.allTimeLow = allTimeLow
    }

    public func syncWalletValues(walletId _: Gemstone.WalletId, period _: Gemstone.ChartPeriod, currency _: Gemstone.Currency) async throws -> GemPortfolioValues {
        try GemPortfolioValues(values: [], allTimeHigh: allTimeHigh?.json(), allTimeLow: allTimeLow?.json())
    }

    public func portfolioData(input _: Gemstone.GemPortfolioDataInput) async throws -> Gemstone.PortfolioData {
        try Primitives.PortfolioData(
            charts: [PortfolioChartData(chartType: .value, values: [])],
            statistics: [allTimeHigh.map { .allTimeHigh($0) }, allTimeLow.map { .allTimeLow($0) }].compactMap(\.self),
            availablePeriods: [.day, .week, .month, .year, .all],
        ).json()
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
    private let stakeBalanceShown: Bool
    private let stakedValue: Gemstone.GemBigInt
    private let rewardsShown: Bool
    private let completionDateShown: Bool
    private let claimable: Bool
    private let explorerAddress: String?
    private let actions: [Gemstone.GemDelegationAction]
    private let validators: [Gemstone.DelegationValidator]

    public init(
        earnData: String = "{}",
        stakeBalanceShown: Bool = false,
        stakedValue: Gemstone.GemBigInt = "0",
        rewardsShown: Bool = false,
        completionDateShown: Bool = false,
        claimable: Bool = false,
        explorerAddress: String? = nil,
        actions: [Gemstone.GemDelegationAction] = [],
        validators: [Gemstone.DelegationValidator] = [],
    ) {
        self.earnData = earnData
        self.stakeBalanceShown = stakeBalanceShown
        self.stakedValue = stakedValue
        self.rewardsShown = rewardsShown
        self.completionDateShown = completionDateShown
        self.claimable = claimable
        self.explorerAddress = explorerAddress
        self.actions = actions
        self.validators = validators
    }

    public func delegationActions(walletType _: Gemstone.GemWalletType, chain _: Gemstone.Chain, provider _: Gemstone.StakeProviderType, state _: Gemstone.DelegationState) -> [Gemstone.GemDelegationAction] {
        actions
    }

    public func canClaimDelegationRewards(walletType _: Gemstone.GemWalletType, chain _: Gemstone.Chain, state _: Gemstone.DelegationState, rewards _: String) -> Bool {
        claimable
    }

    public func validatorExplorerAddress(validator _: Gemstone.DelegationValidator) -> String? {
        explorerAddress
    }

    public func showsCompletionDate(state _: Gemstone.DelegationState) -> Bool {
        completionDateShown
    }

    public func showsRewards(state _: Gemstone.DelegationState, rewards _: String) -> Bool {
        rewardsShown
    }

    public func canClaimStakeRewards(chain _: Gemstone.Chain, rewardsValue _: String) -> Bool {
        claimable
    }

    public func requiresFrozenBalance(chain _: Gemstone.Chain, frozenValue _: String) -> Bool {
        false
    }

    public func recommendedValidatorIds(chain _: Gemstone.Chain) -> [String] {
        validators.map { $0 }
    }

    public func recommendedValidator(chain _: Gemstone.Chain, validators _: [Gemstone.DelegationValidator]) -> Gemstone.DelegationValidator? {
        self.validators.first
    }

    public func selectableValidators(validators _: [Gemstone.DelegationValidator]) -> [Gemstone.DelegationValidator] {
        self.validators
    }

    public func stakedValue(chain _: Gemstone.Chain, balance _: Gemstone.GemStakeBalance) -> Gemstone.GemBigInt {
        stakedValue
    }

    public func showsStakeBalance(chain _: Gemstone.Chain, isStakeEnabled _: Bool, balance _: Gemstone.GemStakeBalance) -> Bool {
        stakeBalanceShown
    }

    public func sync(walletId _: String, chain _: Gemstone.Chain, address _: String) async throws {}

    public func syncEarn(walletId _: String, assetId _: Gemstone.AssetId, address _: String) async throws {}

    public func getEarnData(assetId _: Gemstone.AssetId, address _: String, value _: String, earnType _: Gemstone.EarnType) async throws -> Gemstone.ContractCallData {
        earnData
    }
}

public final class GemTransactionStateServiceMock: GemTransactionStateServiceProtocol, @unchecked Sendable {
    private let store: (any GemTransactionStateStore)?
    private let notificationAsset: Gemstone.Asset?

    public init(store: (any GemTransactionStateStore)? = nil, notificationAsset: Gemstone.Asset? = nil) {
        self.store = store
        self.notificationAsset = notificationAsset
    }

    public func trackPending() async throws {}

    public func track(walletId _: Gemstone.WalletId, transactions _: [Gemstone.Transaction]) async throws {}

    public func stopTracking() {}

    public func addNotificationTransaction(wallet _: Gemstone.Wallet, assetId _: Gemstone.AssetId, transaction: Gemstone.Transaction) async throws -> Gemstone.Asset? {
        if let store {
            try await store.addTransactions(walletId: "", transactions: [transaction])
        }
        return notificationAsset
    }
}

public final class GemBalanceServiceMock: GemBalanceServiceProtocol, @unchecked Sendable {
    private let onUpdate: @Sendable (String, [Gemstone.AssetId]) async -> Void
    private let onSetAssetsEnabled: (@Sendable (String, [Gemstone.AssetId], Bool) async throws -> Void)?
    private let onSetAssetPinned: (@Sendable (String, Gemstone.AssetId, Bool) async throws -> Void)?
    private let assetBalances: [GemAssetBalance]

    public init(
        onUpdate: @escaping @Sendable (String, [Gemstone.AssetId]) async -> Void = { _, _ in },
        onSetAssetsEnabled: (@Sendable (String, [Gemstone.AssetId], Bool) async throws -> Void)? = nil,
        onSetAssetPinned: (@Sendable (String, Gemstone.AssetId, Bool) async throws -> Void)? = nil,
        assetBalances: [GemAssetBalance] = [],
    ) {
        self.onUpdate = onUpdate
        self.onSetAssetsEnabled = onSetAssetsEnabled
        self.onSetAssetPinned = onSetAssetPinned
        self.assetBalances = assetBalances
    }

    public func balances(walletId: String, assetIds: [Gemstone.AssetId]) throws -> [GemAssetBalance] {
        assetBalances.filter { assetIds.contains($0.assetId) }
    }

    public func update(walletId: String, assetIds: [Gemstone.AssetId]) async throws {
        await onUpdate(walletId, assetIds)
    }

    public func setAssetsEnabled(walletId: String, assetIds: [Gemstone.AssetId], enabled: Bool) async throws {
        try await onSetAssetsEnabled?(walletId, assetIds, enabled)
    }

    public func setAssetPinned(walletId: String, assetId: Gemstone.AssetId, pinned: Bool) async throws {
        try await onSetAssetPinned?(walletId, assetId, pinned)
    }
}

public extension GemBalanceServiceProtocol where Self == GemBalanceServiceMock {
    static func mock(
        onSetAssetsEnabled: (@Sendable (String, [Gemstone.AssetId], Bool) async throws -> Void)? = nil,
        onSetAssetPinned: (@Sendable (String, Gemstone.AssetId, Bool) async throws -> Void)? = nil,
    ) -> GemBalanceServiceMock {
        GemBalanceServiceMock(onSetAssetsEnabled: onSetAssetsEnabled, onSetAssetPinned: onSetAssetPinned)
    }
}

public final class GemPerpetualServiceMock: GemPerpetualServiceProtocol, @unchecked Sendable {
    public var autocloseSummary: GemAutocloseSummary?
    public var isPerpetualEnabled = true
    public private(set) var syncMarketsCount = 0
    public private(set) var clearMarketsCount = 0
    public var connectionFailures = 0
    private var updatedAt: Int64?

    public init(marketsUpdatedAt: Int64? = nil) {
        updatedAt = marketsUpdatedAt
    }

    public func marketsUpdatedAt() throws -> Int64? {
        updatedAt
    }

    public func autocloseSummary(data _: Gemstone.PerpetualModifyConfirmData) -> GemAutocloseSummary? {
        autocloseSummary
    }

    public func collateralAssetId(chain _: Gemstone.Chain) -> Gemstone.AssetId? {
        .none
    }

    public func syncEnablement(wallet: Gemstone.Wallet?) async throws -> Bool {
        if isPerpetualEnabled {
            try await syncMarkets(chain: "hypercore")
        } else {
            try await clearMarkets()
        }
        return shouldConnectPerpetuals(wallet: wallet)
    }

    public func shouldConnectPerpetuals(wallet: Gemstone.Wallet?) -> Bool {
        isPerpetualEnabled && (wallet.flatMap { try? Primitives.Wallet($0).hasPerpetualsSupport } ?? false)
    }

    public func syncMarketsIfStale(chain: Gemstone.Chain) async throws -> Bool {
        try await syncMarkets(chain: chain)
        return true
    }

    public func syncMarkets(chain _: Gemstone.Chain) async throws {
        syncMarketsCount += 1
        updatedAt = Int64(Date().timeIntervalSince1970)
    }

    public func clearMarkets() async throws {
        clearMarketsCount += 1
        updatedAt = nil
    }

    public func syncPositions(walletId _: String, chain _: Gemstone.Chain, address _: String) async throws -> Gemstone.PerpetualAccountMode {
        Primitives.PerpetualAccountMode.standard.json()
    }

    public func connection(wallet: Gemstone.Wallet) async throws -> Gemstone.GemPerpetualConnection? {
        if connectionFailures > 0 {
            connectionFailures -= 1
            throw AnyError("connection unavailable")
        }
        guard let account = try Primitives.Wallet(wallet).hyperliquidAccount else { return nil }
        return try Gemstone.GemPerpetualConnection(
            address: account.address,
            mode: Primitives.PerpetualAccountMode.standard.json(),
        )
    }

    public func setPinned(perpetualId _: String, pinned _: Bool) async throws {}

    public func getCandlesticks(chain _: Gemstone.Chain, symbol _: String, period _: Gemstone.ChartPeriod) async throws -> [Gemstone.ChartCandleStick] { [] }

    public func candleInterval(period _: Gemstone.ChartPeriod) -> String { "" }

    public func mergeCandle(candles: [Gemstone.ChartCandleStick], candle: Gemstone.ChartCandleStick) -> [Gemstone.ChartCandleStick] {
        candles + [candle]
    }

    public func getPortfolio(chain _: Gemstone.Chain, address _: String) async throws -> Gemstone.PerpetualPortfolio {
        try Primitives.PerpetualPortfolio(day: nil, week: nil, month: nil, allTime: nil, accountSummary: nil).json()
    }

    public func applySocketMessage(walletId _: String, mode _: Gemstone.PerpetualAccountMode, data _: Data) async throws -> Gemstone.GemPerpetualSocketUpdate {
        .applied
    }

    public func accountMode(walletId _: String, chain _: Gemstone.Chain, address _: String) async throws -> Gemstone.PerpetualAccountMode {
        Primitives.PerpetualAccountMode.standard.json()
    }
}

public final class GemAssetDiscoveryServiceMock: GemAssetDiscoveryServiceProtocol, @unchecked Sendable {
    public init() {}

    public func discover(walletId _: String) async throws -> [Gemstone.AssetId] {
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

    public func visibleBanners(stored: [GemBannerItem], context _: GemBannerContext) -> [GemBannerItem] {
        stored
    }

    public func showsOnboarding(state _: Gemstone.BannerState, isWalletEmpty: Bool) -> Bool {
        isWalletEmpty
    }

    public func close(key: GemBannerKey) async throws {
        closedKeys.append(key)
    }

    public func applyAction(key _: GemBannerKey, action: GemBannerAction) async throws {
        handledActions.append(action)
    }

    public func setup() async throws {}

    public func setupWallet(wallet _: Gemstone.Wallet) async throws {}

    public func bannerContent(event _: Gemstone.BannerEvent, asset _: Gemstone.Asset?) -> GemBannerContent {
        GemBannerContent(icon: .none, title: .none, description: .none)
    }
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
        assets.map { $0.json() }
    }
}

public final class GemStreamSubscriptionServiceMock: GemStreamSubscriptionServiceProtocol, @unchecked Sendable {
    public init() {}

    public func setupAssets(walletId _: Gemstone.WalletId) async throws {}

    public func resubscribe() async throws {}

    public func addPrices(assetIds _: [Gemstone.AssetId]) async throws {}

    public func reset() async {}
}

public final class GemAvatarServiceMock: GemAvatarServiceProtocol, @unchecked Sendable {
    public init() {}

    public func setImage(walletId _: Gemstone.WalletId, image _: Data) async throws {}

    public func setImageUrl(walletId _: Gemstone.WalletId, url _: String) async throws {}

    public func removeImage(walletId _: Gemstone.WalletId) async throws {}
}

public final class GemWalletPreferencesStoreMock: GemWalletPreferencesStore, @unchecked Sendable {
    private let lock = NSLock()
    private var values: [String: String] = [:]

    public init() {}

    public func get(walletId: Gemstone.WalletId, key: String) -> String? {
        lock.withLock { values["\(walletId):\(key)"] }
    }

    public func set(walletId: Gemstone.WalletId, key: String, value: String) throws {
        lock.withLock { values["\(walletId):\(key)"] = value }
    }

    public func deletePreferences(walletId: Gemstone.WalletId) throws {
        lock.withLock { values = values.filter { !$0.key.hasPrefix("\(walletId):") } }
    }
}

public extension GemWalletPreferencesService {
    static func mock() -> GemWalletPreferencesService {
        GemWalletPreferencesService(store: GemWalletPreferencesStoreMock())
    }
}
