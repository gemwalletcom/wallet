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

    public func getCurrency() -> Gemstone.Currency { Primitives.Currency.usd.rawValue }

    public func setCurrency(currency _: Gemstone.Currency) throws {}

    public func setupCurrency(localeCurrency _: String?) throws -> Gemstone.Currency { Primitives.Currency.usd.rawValue }

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
        perpetualEnabled && ((try? Primitives.Wallet(wallet).supportsPerpetuals) ?? false)
    }

    public var collectionsShown = true

    public func showCollections(wallet _: Gemstone.Wallet) -> Bool {
        collectionsShown
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

    public func currency() -> Gemstone.Currency {
        Primitives.Currency.usd.rawValue
    }

    public func setAutoAlert(assetId _: Gemstone.AssetId, enabled isEnabled: Bool) async throws {
        lock.withLock { enabled = isEnabled }
    }

    public func priceAlertId(alert: Gemstone.PriceAlert) -> String {
        (try? Primitives.PriceAlert(alert).id) ?? ""
    }
}

public final class StubAlienProvider: AlienProvider, @unchecked Sendable {
    public init() {}

    public func request(target: AlienTarget) async throws -> AlienResponse {
        throw AnyError("StubAlienProvider does not perform requests")
    }

    public func getEndpoint(chain: Gemstone.Chain) throws -> String {
        throw AnyError("StubAlienProvider has no endpoints")
    }
}

private func contactService() -> GemContactService {
    GemContactService(
        store: GemContactStoreMock(),
        addressStore: GemAddressStoreMock(),
        files: GemFileStoreMock(),
    )
}

public final class GemManageContactServiceMock: GemManageContactServiceProtocol, @unchecked Sendable {
    private let service: GemManageContactService

    public init() {
        service = GemManageContactService(
            contacts: contactService(),
            addresses: GemAddressService(),
            names: GemNameService.mock(),
            chains: GemChainService(),
        )
    }

    public func names() -> GemNameService {
        service.names()
    }

    public func chains() -> GemChainService {
        service.chains()
    }

    public func defaultChain() -> Gemstone.Chain {
        service.defaultChain()
    }

    public func saveContact(input: GemContactInput) async throws -> Gemstone.Contact {
        try await service.saveContact(input: input)
    }

    public func formatAddress(address: String, chain: Gemstone.Chain, style: GemAddressFormatStyle) -> String {
        service.formatAddress(address: address, chain: chain, style: style)
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

public final class GemAmountServiceMock: GemAmountServiceProtocol, @unchecked Sendable {
    public init() {}

    public func currency() -> Gemstone.Currency { Primitives.Currency.usd.rawValue }

    public func earnData(assetId _: Gemstone.AssetId, address _: String, value _: String, earnType _: Gemstone.EarnType) async throws -> Gemstone.ContractCallData {
        throw AnyError("not stubbed")
    }

    public func perpetualLeverage() -> UInt8 { 5 }

    public func perpetualStopLossPercent() -> UInt8 { 0 }

    public func perpetualTakeProfitPercent() -> UInt8 { 0 }
}

public final class GemFiatQuoteServiceMock: GemFiatQuoteServiceProtocol, @unchecked Sendable {
    private let quotes: [Primitives.FiatQuote]
    private let check: @Sendable (Primitives.FiatQuote?) -> GemFiatAmountCheck

    public init(quotes: [Primitives.FiatQuote] = [], check: @escaping @Sendable (Primitives.FiatQuote?) -> GemFiatAmountCheck = { _ in .valid }) {
        self.quotes = quotes
        self.check = check
    }

    public func currency() -> Gemstone.Currency { Primitives.Currency.usd.rawValue }

    public func config() -> Gemstone.FiatConfig {
        Gemstone.FiatConfig(defaultBuyAmount: 50, defaultSellAmount: 100, minimumAmount: 5, maximumAmount: 10000, randomMaxAmount: 1000, suggestedAmounts: [100, 250], insufficientNetworkFeeBuyAmount: 10)
    }

    public func defaultAmount(quoteType: Gemstone.FiatQuoteType) -> UInt32 {
        (try? Primitives.FiatQuoteType(quoteType)) == .sell ? 100 : 50
    }

    public func randomAmount() -> UInt32 { 50 }

    public func amountCheck(quoteType _: Gemstone.FiatQuoteType, amount _: Double, quote: Gemstone.FiatQuote?, available _: GemBigUint) -> GemFiatAmountCheck {
        check(quote.flatMap { try? Primitives.FiatQuote($0) })
    }

    public func quoteDebounceMilliseconds() -> UInt64 { 250 }

    public func quoteRefreshIntervalMilliseconds() -> UInt64 { 300_000 }

    public func syncTransactions(walletId _: Gemstone.WalletId) async throws {}

    public func quotes(walletId _: Gemstone.WalletId, quoteType _: Gemstone.FiatQuoteType, assetId _: Gemstone.AssetId, amount _: Double) async throws -> [Gemstone.FiatQuote] {
        quotes.map { $0.json() }
    }

    public func quoteUrl(walletId _: Gemstone.WalletId, assetId _: Gemstone.AssetId, quoteId _: String) async throws -> Gemstone.FiatQuoteUrl {
        throw AnyError("not stubbed")
    }
}

public extension GemPaymentService {
    static func mock() -> GemPaymentService {
        GemPaymentService(provider: StubAlienProvider())
    }
}

public extension GemNameService {
    static func mock() -> GemNameService {
        GemNameService(
            api: GemDeviceApiClient(
                provider: StubAlienProvider(),
                baseUrl: "https://localhost",
                deviceKey: GemDeviceKeyService(store: GemSecureStoreMock()),
            ),
            store: GemAddressStoreMock(),
        )
    }
}

public final class GemNameServiceMock: GemNameServiceProtocol, @unchecked Sendable {
    private let rules = GemNameService.mock()
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

    public func validateRecipient(chain: Gemstone.Chain, input: String, nameRecord: Gemstone.NameRecord?) -> GemRecipientValidation {
        rules.validateRecipient(chain: chain, input: input, nameRecord: nameRecord)
    }

    public func recipient(chain: Gemstone.Chain, input: String, nameRecord: Gemstone.NameRecord?, memo: String?, references: [String]) throws -> GemRecipient {
        try rules.recipient(chain: chain, input: input, nameRecord: nameRecord, memo: memo, references: references)
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
    private let rewardsShown: Bool
    private let completionDateShown: Bool
    private let claimable: Bool
    private let explorerAddress: String?
    private let actions: [Gemstone.GemDelegationAction]
    private let validators: [Gemstone.DelegationValidator]

    public init(
        earnData: String = "{}",
        rewardsShown: Bool = false,
        completionDateShown: Bool = false,
        claimable: Bool = false,
        explorerAddress: String? = nil,
        actions: [Gemstone.GemDelegationAction] = [],
        validators: [Gemstone.DelegationValidator] = [],
    ) {
        self.earnData = earnData
        self.rewardsShown = rewardsShown
        self.completionDateShown = completionDateShown
        self.claimable = claimable
        self.explorerAddress = explorerAddress
        self.actions = actions
        self.validators = validators
    }

    public func delegationActions(walletType _: Gemstone.WalletType, chain _: Gemstone.Chain, provider _: Gemstone.StakeProviderType, state _: Gemstone.DelegationState) -> [Gemstone.GemDelegationAction] {
        actions
    }

    public func canClaimDelegationRewards(walletType _: Gemstone.WalletType, chain _: Gemstone.Chain, state _: Gemstone.DelegationState, rewards _: String) -> Bool {
        claimable
    }

    public func currency() -> Gemstone.Currency {
        Primitives.Currency.usd.rawValue
    }

    public func validatorUrl(validator _: Gemstone.DelegationValidator) -> GemBlockExplorerLink? {
        explorerAddress.map { GemBlockExplorerLink(name: "MockExplorer", link: "https://explorer.mock/validator/\($0)") }
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

    public func syncEnablement(wallet: Gemstone.Wallet?, trigger: Gemstone.GemMarketsRefreshTrigger) async throws -> Bool {
        if isPerpetualEnabled {
            _ = try await syncMarketsIfNeeded(chain: "hypercore", trigger: trigger)
        } else {
            try await clearMarkets()
        }
        return shouldConnectPerpetuals(wallet: wallet)
    }

    public func shouldConnectPerpetuals(wallet: Gemstone.Wallet?) -> Bool {
        isPerpetualEnabled && (wallet.flatMap { try? Primitives.Wallet($0).supportsPerpetuals } ?? false)
    }

    public func syncMarketsIfNeeded(chain: Gemstone.Chain, trigger: Gemstone.GemMarketsRefreshTrigger) async throws -> Bool {
        if trigger == .scheduled, updatedAt != nil {
            return false
        }
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

public extension GemTransactionDetailsService {
    static func mock() -> GemTransactionDetailsService {
        let preferences = GemPreferencesService(store: GemPreferencesStoreMock())
        return GemTransactionDetailsService(explorer: GemExplorerService(preferences: preferences), preferences: preferences)
    }
}

public final class GemWalletHomeServiceMock: GemWalletHomeServiceProtocol, @unchecked Sendable {
    public private(set) var handledActions: [GemBannerAction] = []
    public private(set) var pinned: [(assetId: Gemstone.AssetId, pinned: Bool)] = []
    public private(set) var enabled: [(assetIds: [Gemstone.AssetId], enabled: Bool)] = []
    public var showsLoading = false

    public init() {}

    public func currency() -> Gemstone.Currency {
        Primitives.Currency.usd.rawValue
    }

    public func updateBalances(walletId _: Gemstone.WalletId, assetIds _: [Gemstone.AssetId]) async throws {}

    public func includesPerpetualCollateral(walletId _: Gemstone.WalletId) -> Bool {
        false
    }

    public func showsInitialLoading(walletId _: Gemstone.WalletId) throws -> Bool {
        showsLoading
    }

    public func refresh(walletId _: Gemstone.WalletId, assetIds _: [Gemstone.AssetId]) async throws {}

    public func setAssetPinned(walletId _: Gemstone.WalletId, assetId: Gemstone.AssetId, pinned isPinned: Bool) async throws {
        pinned.append((assetId, isPinned))
    }

    public func setAssetsEnabled(walletId _: Gemstone.WalletId, assetIds: [Gemstone.AssetId], enabled isEnabled: Bool) async throws {
        enabled.append((assetIds, isEnabled))
    }

    public func bannerContent(event _: Gemstone.BannerEvent, asset _: Gemstone.Asset?) -> GemBannerContent {
        GemBannerContent(icon: .none, title: .none, description: .none)
    }

    public func applyBannerAction(key _: GemBannerKey, action: GemBannerAction) async throws {
        handledActions.append(action)
    }
}

public final class GemCurrencyServiceMock: GemCurrencyServiceProtocol, @unchecked Sendable {
    public private(set) var setCurrencies: [Gemstone.Currency] = []
    private let error: Error?

    public init(error: Error? = nil) {
        self.error = error
    }

    public func currency() -> Gemstone.Currency {
        setCurrencies.last ?? Primitives.Currency.usd.rawValue
    }

    public func setCurrency(currency: Gemstone.Currency) async throws {
        if let error { throw error }
        setCurrencies.append(currency)
    }
}

public final class GemBannerServiceMock: GemBannerServiceProtocol, @unchecked Sendable {
    public private(set) var closedKeys: [GemBannerKey] = []
    public private(set) var handledActions: [GemBannerAction] = []

    public init() {}

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

public extension Gemstone.GemFeeAsset {
    static func mock(
        asset: Primitives.Asset,
        balance: Gemstone.GemAssetBalance? = nil,
        price: Gemstone.GemAssetPrice? = nil,
    ) -> Gemstone.GemFeeAsset {
        Gemstone.GemFeeAsset(
            asset: asset.map(),
            balance: balance ?? Gemstone.GemAssetBalance(
                assetId: asset.id.identifier,
                available: "0",
                frozen: "0",
                locked: "0",
                staked: "0",
                pending: "0",
                pendingUnconfirmed: "0",
                rewards: "0",
                reserved: "0",
                withdrawable: "0",
                earn: "0",
                metadata: nil,
            ),
            price: price,
        )
    }
}

private extension Primitives.Wallet {
    var supportsPerpetuals: Bool {
        isMultiCoins && hyperliquidAccount != nil
    }
}
