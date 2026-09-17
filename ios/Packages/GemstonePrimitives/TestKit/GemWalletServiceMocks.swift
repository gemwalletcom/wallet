// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import struct Gemstone.GemPriceAlertSession
import enum Gemstone.GemNameInputStep

public actor GemDeviceServiceMock: GemDeviceServiceProtocol {
    private let syncError: Error?
    public private(set) var synchronizeIfNeededCalls = 0

    public init(syncError: Error? = nil) {
        self.syncError = syncError
    }

    public func synchronizeIfNeeded() async throws {
        synchronizeIfNeededCalls += 1
        if let syncError {
            throw syncError
        }
    }
}

public final class GemPreferencesServiceMock: GemPreferencesServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var priceAlertsEnabled: Bool
    private var skippedAppVersion: String?

    public init(priceAlertsEnabled: Bool = false) {
        self.priceAlertsEnabled = priceAlertsEnabled
    }

    public func setPriceAlertsEnabled(enabled: Bool) throws {
        lock.withLock { priceAlertsEnabled = enabled }
    }

    public func getCurrency() -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func setCurrency(currency _: Gemstone.Currency) throws {}

    public func setupCurrency(localeCurrency _: String?) throws -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func getChartPeriod() -> Gemstone.ChartPeriod {
        Primitives.ChartPeriod.day.toGem()
    }

    public func setChartPeriod(period _: Gemstone.ChartPeriod) throws {}

    public func isPushNotificationsEnabled() -> Bool {
        false
    }

    public func setPushNotificationsEnabled(enabled _: Bool) throws {}

    private var perpetualEnabled = false
    private var hideBalanceEnabled = false
    private var developerEnabled = false
    private var acceptTermsCompleted = false
    private var appearance: Gemstone.Appearance = .system

    public func isPerpetualEnabled() -> Bool {
        perpetualEnabled
    }

    public func setPerpetualEnabled(enabled: Bool) throws {
        perpetualEnabled = enabled
    }

    public func showPerpetuals(walletType _: Gemstone.WalletType, chains _: [Gemstone.Chain]) -> Bool {
        perpetualEnabled
    }

    public var collectionsShown = true

    public func showCollections(walletType _: Gemstone.WalletType, chains _: [Gemstone.Chain]) -> Bool {
        collectionsShown
    }

    public func isHideBalanceEnabled() -> Bool {
        hideBalanceEnabled
    }

    public func setHideBalanceEnabled(enabled: Bool) throws {
        hideBalanceEnabled = enabled
    }

    public func isDeveloperEnabled() -> Bool {
        developerEnabled
    }

    public func setDeveloperEnabled(enabled: Bool) throws {
        developerEnabled = enabled
    }

    public func isAcceptTermsCompleted() -> Bool {
        acceptTermsCompleted
    }

    public func setAcceptTermsCompleted() throws {
        acceptTermsCompleted = true
    }

    public func getAppearance() -> Gemstone.Appearance {
        appearance
    }

    public func setAppearance(appearance: Gemstone.Appearance) throws {
        self.appearance = appearance
    }

    public func incrementLaunchesCount() throws -> UInt32 {
        1
    }

    public func shouldRequestReview() -> Bool {
        false
    }

    public func setRateApplicationShown() throws {}

    public func notificationPrompt(isGranted: Bool) -> Gemstone.GemNotificationPrompt {
        isGranted ? .enable : .request
    }

    public func shouldAskNotifications() -> Bool {
        false
    }

    public func setNotificationsAsked() throws {}

    public func clear() throws {}
}

public final class GemStreamServiceMock: GemStreamServiceProtocol, @unchecked Sendable {
    private let prepare: @Sendable () async throws -> Bool
    private let onConnected: @Sendable () async throws -> Void
    private let onDisconnected: @Sendable () async -> Void
    private let onEvent: @Sendable (String) async throws -> GemStreamEvent
    private let onSession: @Sendable () async throws -> Void

    public init(
        prepare: @escaping @Sendable () async throws -> Bool = { true },
        onConnected: @escaping @Sendable () async throws -> Void = {},
        onDisconnected: @escaping @Sendable () async -> Void = {},
        onEvent: @escaping @Sendable (String) async throws -> GemStreamEvent = { _ in .prices(prices: 0, rates: 0) },
        onSession: @escaping @Sendable () async throws -> Void = {},
    ) {
        self.prepare = prepare
        self.onConnected = onConnected
        self.onDisconnected = onDisconnected
        self.onEvent = onEvent
        self.onSession = onSession
    }

    public func updateSession() async throws {
        try await onSession()
    }

    public func prepareConnection() async throws -> Bool {
        try await prepare()
    }

    public func connected() async throws {
        try await onConnected()
    }

    public func disconnected() async {
        await onDisconnected()
    }

    public func handle(event: String) async throws -> GemStreamEvent {
        try await onEvent(event)
    }
}

public final class GemPortfolioServiceMock: GemPortfolioServiceProtocol, @unchecked Sendable {
    private let allTimeHigh: Primitives.ChartValuePercentage?
    private let allTimeLow: Primitives.ChartValuePercentage?

    public init(allTimeHigh: Primitives.ChartValuePercentage? = nil, allTimeLow: Primitives.ChartValuePercentage? = nil) {
        self.allTimeHigh = allTimeHigh
        self.allTimeLow = allTimeLow
    }

    public func currency(portfolioType _: Gemstone.PortfolioType) -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func portfolioData(wallet _: Gemstone.Wallet, portfolioType _: Gemstone.PortfolioType, period _: Gemstone.ChartPeriod) async throws -> Gemstone.PortfolioData {
        Gemstone.PortfolioData(
            charts: [Gemstone.PortfolioChartData(chartType: .value, values: [])],
            statistics: [allTimeHigh.map { .allTimeHigh(value: $0.toGem()) }, allTimeLow.map { .allTimeLow(value: $0.toGem()) }].compactMap(\.self),
            availablePeriods: [.day, .week, .month, .year, .all],
        )
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

    public func balances(walletId _: String, assetIds: [Gemstone.AssetId]) throws -> [GemAssetBalance] {
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

public final class GemWalletHomeServiceMock: GemWalletHomeServiceProtocol, @unchecked Sendable {
    public private(set) var closedKeys: [GemBannerKey] = []
    public private(set) var pinned: [(assetId: Gemstone.AssetId, pinned: Bool)] = []
    public private(set) var enabled: [(assetIds: [Gemstone.AssetId], enabled: Bool)] = []
    public var showsLoading = false

    public init() {}

    public func getCurrency() -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func assetRow() -> Gemstone.GemAssetRow {
        Gemstone.GemAssetRow(title: .asset, showsSymbol: false, subtitle: .price, trailing: .balance)
    }

    public func viewState(wallet: Gemstone.Wallet, balances: [Gemstone.AssetFiatValue], perpetual: Gemstone.PerpetualBalance?, banners: [Gemstone.Banner], isWalletEmpty: Bool) -> GemWalletHomeViewState {
        let value = balances.reduce(0.0) { $0 + $1.amount * $1.price } + (perpetual.map { $0.available + $0.reserved } ?? 0)
        let total = Gemstone.TotalFiatValue(value: value, pnlAmount: 0, pnlPercentage: 0)
        let isEnabled = !banners.contains { $0.event == .accountBlockedMultiSignature }
        let context = GemBannerContext(
            wallet: wallet,
            asset: nil,
            isStakeable: false,
            hasStakeBalance: false,
            hasAvailableBalance: false,
            isAssetActivated: true,
            assetRankScore: nil,
            isWalletEmpty: isWalletEmpty,
        )
        return GemWalletHomeViewState(
            totalValue: total,
            showsPnl: total.value > 0 && total.pnlAmount != 0,
            headerActions: .buttons(buttons: [GemHeaderButtonKind.send, .receive, .buy].map { GemHeaderButton(kind: $0, isEnabled: isEnabled) }),
            showCollections: false,
            visibleBanners: context.visibleBanners(stored: banners),
        )
    }

    public func updateBalances(assetIds _: [Gemstone.AssetId]) async throws {}

    public func showsInitialLoading() throws -> Bool {
        showsLoading
    }

    public func refresh() async throws {}

    public func setAssetPinned(assetId: Gemstone.AssetId, pinned isPinned: Bool) async throws {
        pinned.append((assetId, isPinned))
    }

    public func setAssetsEnabled(assetIds: [Gemstone.AssetId], enabled isEnabled: Bool) async throws {
        enabled.append((assetIds, isEnabled))
    }

    public func bannerContent(event _: Gemstone.BannerEvent, asset _: Gemstone.Asset?) -> GemBannerContent {
        GemBannerContent(icon: .none, title: .none, description: .none, link: .none)
    }

    public func closeBanner(key: GemBannerKey) async throws {
        closedKeys.append(key)
    }
}

public final class GemCurrencyServiceMock: GemCurrencyServiceProtocol, @unchecked Sendable {
    public private(set) var setCurrencies: [Gemstone.Currency] = []
    private let error: Error?
    private let flag: String

    public init(flag: String = "🇺🇸", error: Error? = nil) {
        self.flag = flag
        self.error = error
    }

    public func getCurrency() -> Gemstone.Currency {
        setCurrencies.last ?? Primitives.Currency.usd.toGem()
    }

    public func setCurrency(currency: Gemstone.Currency) async throws {
        if let error { throw error }
        setCurrencies.append(currency)
    }

    public func currencies(locale _: Gemstone.Currency?) -> GemCurrencies {
        let selected = GemCurrencyRow(currency: getCurrency(), flag: flag)
        return GemCurrencies(selected: selected, recommended: [selected], other: [])
    }
}

public final class GemBannerServiceMock: GemBannerServiceProtocol, @unchecked Sendable {
    public private(set) var closedKeys: [GemBannerKey] = []

    public init() {}

    public func close(key: GemBannerKey) async throws {
        closedKeys.append(key)
    }

    public func setup() async throws {}

    public func setupWallet(wallet _: Gemstone.Wallet) async throws {}

    public func bannerContent(event _: Gemstone.BannerEvent, asset _: Gemstone.Asset?) -> GemBannerContent {
        GemBannerContent(icon: .none, title: .none, description: .none, link: .none)
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
        assets.map { $0.toGem() }
    }
}

public final class GemSettingsServiceMock: GemSettingsServiceProtocol, @unchecked Sendable {
    public var sectionsValue: [GemSettingsSection] = []
    public var securitySectionsValue: [GemSecuritySection] = []
    public var perpetualDefaults = GemPerpetualDefaults(leverage: 3, takeProfitPercent: 25, stopLossPercent: 10)
    public var preferencesSections: [GemPreferencesSection] = []
    public var setDefaultsError: Error?

    public private(set) var storedDefaults: [GemPerpetualDefaults] = []
    public private(set) var securitySectionsCalls: [Bool] = []
    public private(set) var perpetualsEnabledCalls: [Bool] = []

    public init() {}

    public func preferences(currency: Gemstone.Currency, perpetualsEnabled: Bool) -> GemPreferencesState {
        perpetualsEnabledCalls.append(perpetualsEnabled)
        return GemPreferencesState(
            currency: GemCurrencyRow(currency: currency, flag: "🇺🇸"),
            sections: preferencesSections,
            perpetualDefaults: perpetualDefaults,
        )
    }

    public func sections(wallets _: [Gemstone.Wallet], notificationsAvailable _: Bool, walletConnectAvailable _: Bool) -> [GemSettingsSection] {
        sectionsValue
    }

    public func securitySections(authenticationEnabled: Bool) -> [GemSecuritySection] {
        securitySectionsCalls.append(authenticationEnabled)
        return securitySectionsValue
    }

    public func setPerpetualDefaults(defaults: GemPerpetualDefaults) throws {
        if let setDefaultsError { throw setDefaultsError }
        storedDefaults.append(defaults)
        perpetualDefaults = defaults
    }
}

public final class GemAppUpdateServiceMock: GemAppUpdateServiceProtocol, @unchecked Sendable {
    public var newestValue: Gemstone.Release?
    public var newestError: Error?

    public private(set) var skippedVersions: [String] = []

    public init(newest: Gemstone.Release? = nil) {
        newestValue = newest
    }

    public func check(store _: Gemstone.PlatformStore, currentVersion _: String) async throws -> Gemstone.Release? {
        if let newestError { throw newestError }
        return newestValue
    }

    public func isVersionHigher(new: String, current: String) -> Bool {
        new.compare(current, options: .numeric) == .orderedDescending
    }

    public func newest(store _: Gemstone.PlatformStore, currentVersion _: String) async throws -> Gemstone.Release? {
        if let newestError { throw newestError }
        return newestValue
    }

    public func skip(version: String) throws {
        skippedVersions.append(version)
    }
}

extension Primitives.Wallet {
    var hyperliquidAccount: Primitives.Account? {
        accounts.first { $0.chain == .hyperCore }
    }
}
