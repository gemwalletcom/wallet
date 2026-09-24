// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import enum Gemstone.GemNameInputStep
import struct Gemstone.GemPriceAlertSession
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public actor GemDeviceServiceMock: GemDeviceServiceProtocol {
    private let syncError: Error?
    private let onSynchronize: (@Sendable () -> Void)?
    public private(set) var synchronizeIfNeededCalls = 0

    public init(syncError: Error? = nil, onSynchronize: (@Sendable () -> Void)? = nil) {
        self.syncError = syncError
        self.onSynchronize = onSynchronize
    }

    public func synchronizeIfNeeded() async throws {
        synchronizeIfNeededCalls += 1
        onSynchronize?()
        if let syncError {
            throw syncError
        }
    }
}

public final class GemPreferencesServiceMock: GemPreferencesServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var priceAlertsEnabled: Bool
    private var skippedAppVersion: String?

    public init(priceAlertsEnabled: Bool = false, perpetualEnabled: Bool = false) {
        self.priceAlertsEnabled = priceAlertsEnabled
        self.perpetualEnabled = perpetualEnabled
    }

    public func setPriceAlertsEnabled(enabled: Bool) throws {
        lock.withLock { priceAlertsEnabled = enabled }
    }

    public func setObserver(observer _: any GemPreferencesObserver) {}

    public func getCurrency() -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func setCurrency(currency _: Gemstone.Currency) throws {}

    public func setupCurrency(localeCurrency _: String?) throws -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func isPushNotificationsEnabled() -> Bool {
        false
    }

    public func setPushNotificationsEnabled(enabled _: Bool) throws {}

    private var perpetualEnabled: Bool
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
}

public final class GemStreamServiceMock: GemStreamServiceProtocol, @unchecked Sendable {
    private let prepare: @Sendable () async throws -> Bool
    private let onConnected: @Sendable () async throws -> Void
    private let onDisconnected: @Sendable () async -> Void
    private let onEvent: @Sendable (String) async throws -> GemStreamEvent
    private let onSession: @Sendable () async throws -> Void
    private let onSync: @Sendable (GemStreamEvent) async throws -> Void

    public init(
        prepare: @escaping @Sendable () async throws -> Bool = { true },
        onConnected: @escaping @Sendable () async throws -> Void = {},
        onDisconnected: @escaping @Sendable () async -> Void = {},
        onEvent: @escaping @Sendable (String) async throws -> GemStreamEvent = { _ in .prices(prices: 0, rates: 0) },
        onSession: @escaping @Sendable () async throws -> Void = {},
        onSync: @escaping @Sendable (GemStreamEvent) async throws -> Void = { _ in },
    ) {
        self.prepare = prepare
        self.onConnected = onConnected
        self.onDisconnected = onDisconnected
        self.onEvent = onEvent
        self.onSession = onSession
        self.onSync = onSync
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

    public func decodeEvent(event: String) async throws -> GemStreamEvent {
        try await onEvent(event)
    }

    public func sync(event: GemStreamEvent) async throws {
        try await onSync(event)
    }
}

public final class GemPortfolioServiceMock: GemPortfolioServiceProtocol, @unchecked Sendable {
    public var dataForType: (Gemstone.PortfolioType) -> Gemstone.PortfolioData
    public var error: GemServiceError?
    public var perpetualsShown = false

    public private(set) var requests: [GemPortfolioRequest] = []

    public init() {
        dataForType = { type in
            switch type {
            case .wallet: .mockWallet()
            case .perpetuals: .mockPerpetual()
            }
        }
    }

    public func currency(portfolioType _: Gemstone.PortfolioType) -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func showPerpetuals(walletType _: Gemstone.WalletType, chains _: [Gemstone.Chain]) -> Bool {
        perpetualsShown
    }

    public func portfolioData(wallet _: Gemstone.Wallet, portfolioType: Gemstone.PortfolioType, period _: Gemstone.ChartPeriod) async throws -> Gemstone.PortfolioData {
        if let error {
            throw error
        }
        return dataForType(portfolioType)
    }

    public func refresh(wallet _: Gemstone.Wallet, request: GemPortfolioRequest) async -> GemPortfolioResult {
        requests.append(request)
        if let error {
            return GemPortfolioResult(request: request, state: .error(error: error), data: nil)
        }
        return GemPortfolioResult(request: request, state: .data, data: dataForType(request.portfolioType))
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

    public func assetRowStyle() -> Gemstone.GemAssetRowStyle {
        Gemstone.GemAssetRowStyle(title: .asset, showsSymbol: false, subtitle: .price, trailing: .balance)
    }

    public func viewState(wallet: Gemstone.Wallet, balances: [Gemstone.AssetFiatValue], perpetual: Gemstone.GemPerpetualCollateral?, banners: [Gemstone.Banner]) -> GemWalletHomeViewState {
        let collateral: Double = perpetual.map { ($0.balance.available + $0.balance.reserved) * $0.price } ?? 0
        let value = balances.reduce(0.0) { $0 + $1.amount * $1.price } + collateral
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
            isWalletEmpty: balances.allSatisfy { $0.amount == 0 },
        )
        let showsPnl = total.value > 0 && total.pnlAmount != 0
        return GemWalletHomeViewState(
            totalValue: total,
            total: formattedCurrency(value: total.value, code: Currency.usd.rawValue, style: .fiat),
            pnl: showsPnl ? .pnl(
                amount: formattedSignedCurrency(value: total.pnlAmount, code: Currency.usd.rawValue, style: .fiat),
                percent: formattedPercentage(value: total.pnlPercentage, style: .unsigned),
            ) : nil,
            pnlTone: valueTone(value: total.pnlAmount),
            headerActions: .buttons(buttons: [GemHeaderButtonKind.send, .receive, .buy].map { GemHeaderButton(kind: $0, isEnabled: isEnabled) }),
            showCollections: false,
            showsPerpetuals: false,
            visibleBanners: context.visibleBanners(stored: banners),
        )
    }

    public func updateBalances(assetIds _: [Gemstone.AssetId]) async throws {}

    public func showsInitialLoading() -> Bool {
        showsLoading
    }

    public func refresh() async throws {}

    public func setAssetPinned(assetId: Gemstone.AssetId, pinned isPinned: Bool) async throws {
        pinned.append((assetId, isPinned))
    }

    public func setAssetsEnabled(assetIds: [Gemstone.AssetId], enabled isEnabled: Bool) async throws {
        enabled.append((assetIds, isEnabled))
    }

    public func closeBanner(key: GemBannerKey) async throws {
        closedKeys.append(key)
    }
}

public final class GemCurrencyServiceMock: GemCurrencyServiceProtocol, @unchecked Sendable {
    public private(set) var setCurrencies: [Gemstone.Currency] = []
    public private(set) var queries: [String] = []
    public var sectionsValue: [GemCurrencySection] = []
    private let preferencesService: any GemPreferencesServiceProtocol
    private let error: Error?

    public init(
        preferencesService: any GemPreferencesServiceProtocol,
        error: Error? = nil,
    ) {
        self.preferencesService = preferencesService
        self.error = error
    }

    public func setCurrency(currency: Gemstone.Currency) async throws {
        if let error {
            throw error
        }
        setCurrencies.append(currency)
        try preferencesService.setCurrency(currency: currency)
    }

    public func sections(currency _: Gemstone.Currency, locale _: Gemstone.Currency?, query: String, localizedNames _: [String: String]) -> [GemCurrencySection] {
        queries.append(query)
        return sectionsValue
    }
}

public final class GemSettingsServiceMock: GemSettingsServiceProtocol, @unchecked Sendable {
    public var sectionsValue: [GemListSection] = []
    public var securitySectionsValue: [GemListSection] = []
    public var perpetualDefaultsValue = GemPerpetualDefaults(leverage: 3, takeProfitPercent: 25, stopLossPercent: 10)
    public var preferencesSectionsValue: [GemListSection] = []
    public var pickersValue = GemSettingsService(preferences: GemPreferencesService(store: GemPreferencesStoreMock())).perpetualPickers()
    public var setDefaultsError: Error?

    public private(set) var storedDefaults: [GemPerpetualDefaults] = []
    public private(set) var securitySectionsCalls: [GemSecurityInput] = []
    public private(set) var preferencesInputs: [GemPreferencesInput] = []

    public init() {}

    public func preferencesSections(input: GemPreferencesInput) -> [GemListSection] {
        preferencesInputs.append(input)
        return preferencesSectionsValue
    }

    public func perpetualDefaults() -> GemPerpetualDefaults {
        perpetualDefaultsValue
    }

    public func perpetualPickers() -> GemPerpetualPickers {
        pickersValue
    }

    public func sections(wallets _: [Gemstone.Wallet], notificationsAvailable _: Bool, walletConnectAvailable _: Bool) -> [GemListSection] {
        sectionsValue
    }

    public func securitySections(input: GemSecurityInput) -> [GemListSection] {
        securitySectionsCalls.append(input)
        return securitySectionsValue
    }

    public func setPerpetualDefaults(defaults: GemPerpetualDefaults) throws {
        if let setDefaultsError {
            throw setDefaultsError
        }
        storedDefaults.append(defaults)
        perpetualDefaultsValue = defaults
    }
}

public final class GemAppUpdateServiceMock: GemAppUpdateServiceProtocol, @unchecked Sendable {
    public var newestValue: Gemstone.Release?
    public var newestError: Error?

    public private(set) var skippedVersions: [String] = []

    public init(newest: Gemstone.Release? = nil) {
        newestValue = newest
    }

    public func check(store _: Gemstone.PlatformStore, currentVersion _: String) async throws -> GemAppUpdateOffer? {
        if let newestError {
            throw newestError
        }
        return newestValue.map { GemAppUpdateOffer(version: $0.version, canSkip: !$0.upgradeRequired) }
    }

    public func isVersionHigher(new: String, current: String) -> Bool {
        new.compare(current, options: .numeric) == .orderedDescending
    }

    public func newest(store _: Gemstone.PlatformStore, currentVersion _: String) async throws -> Gemstone.Release? {
        if let newestError {
            throw newestError
        }
        return newestValue
    }

    public func skip(offer: GemAppUpdateOffer) throws {
        skippedVersions.append(offer.version)
    }
}

extension Primitives.Wallet {
    var hyperliquidAccount: Primitives.Account? {
        accounts.first { $0.chain == .hyperCore }
    }
}
