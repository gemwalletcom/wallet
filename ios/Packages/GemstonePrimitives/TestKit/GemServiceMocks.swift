// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitives
import Primitives

public actor GemDeviceServiceMock: GemDeviceServiceProtocol {
    private let delay: Duration?
    private let isRegisteredResult: Bool
    private let getDeviceResult: Primitives.Device?
    private let token: Primitives.DeviceToken

    public private(set) var isRegisteredCalls = 0
    public private(set) var getDeviceCalls = 0
    public private(set) var addDeviceCalls = 0
    public private(set) var updateDeviceCalls = 0
    public private(set) var getTokenCalls = 0

    public init(
        delay: Duration? = nil,
        isRegistered: Bool = true,
        getDeviceResult: Primitives.Device? = nil,
        token: Primitives.DeviceToken = .init(token: "", expiresAt: 0),
    ) {
        self.delay = delay
        isRegisteredResult = isRegistered
        self.getDeviceResult = getDeviceResult
        self.token = token
    }

    public func getDevice() async throws -> Gemstone.Device? {
        getDeviceCalls += 1
        return try getDeviceResult?.json()
    }

    public func addDevice(device: Gemstone.Device) async throws -> Gemstone.Device {
        addDeviceCalls += 1
        try await sleepIfNeeded()
        return device
    }

    public func updateDevice(device: Gemstone.Device) async throws -> Gemstone.Device {
        updateDeviceCalls += 1
        try await sleepIfNeeded()
        return device
    }

    public func isRegistered() async throws -> Bool {
        isRegisteredCalls += 1
        return isRegisteredResult
    }

    public func getToken() async throws -> Gemstone.DeviceToken {
        getTokenCalls += 1
        return try token.json()
    }

    private func sleepIfNeeded() async throws {
        if let delay {
            try await Task.sleep(for: delay)
        }
    }
}

public actor GemSubscriptionServiceMock: GemSubscriptionServiceProtocol {
    private let delay: Duration?
    private let subscriptions: [Primitives.WalletSubscriptionChains]
    private let getSubscriptionsError: Error?

    public private(set) var getSubscriptionsCalls = 0

    public init(
        delay: Duration? = nil,
        subscriptions: [Primitives.WalletSubscriptionChains] = [],
        getSubscriptionsError: Error? = nil,
    ) {
        self.delay = delay
        self.subscriptions = subscriptions
        self.getSubscriptionsError = getSubscriptionsError
    }

    public func getSubscriptions() async throws -> [Gemstone.WalletSubscriptionChains] {
        getSubscriptionsCalls += 1
        if let delay {
            try await Task.sleep(for: delay)
        }
        if let getSubscriptionsError {
            throw getSubscriptionsError
        }
        return try subscriptions.map { try $0.json() }
    }

    public func addSubscriptions(subscriptions _: [Gemstone.WalletSubscription]) async throws {}

    public func deleteSubscriptions(subscriptions _: [Gemstone.WalletSubscriptionChains]) async throws {}
}

public final class GemPriceAlertServiceMock: GemPriceAlertServiceProtocol, @unchecked Sendable {
    private let priceAlerts: [Primitives.PriceAlert]

    public init(priceAlerts: [Primitives.PriceAlert] = []) {
        self.priceAlerts = priceAlerts
    }

    public func getPriceAlerts(assetId _: String?) async throws -> [Gemstone.PriceAlert] {
        try priceAlerts.map { try $0.json() }
    }

    public func addPriceAlerts(alerts _: [Gemstone.PriceAlert]) async throws {}

    public func deletePriceAlerts(alerts _: [Gemstone.PriceAlert]) async throws {}
}

public final class GemTransactionsServiceMock: GemTransactionsServiceProtocol, @unchecked Sendable {
    private let lock = NSLock()
    private var walletTransactionsResponse: Primitives.TransactionsResponse
    private var assetTransactionsResponse: Primitives.TransactionsResponse
    private let assetsList: [Primitives.AssetId]

    public init(
        walletTransactionsResponse: Primitives.TransactionsResponse = TransactionsResponse(transactions: [], addressNames: []),
        assetTransactionsResponse: Primitives.TransactionsResponse = TransactionsResponse(transactions: [], addressNames: []),
        assetsList: [Primitives.AssetId] = [],
    ) {
        self.walletTransactionsResponse = walletTransactionsResponse
        self.assetTransactionsResponse = assetTransactionsResponse
        self.assetsList = assetsList
    }

    public func getTransactions(walletId _: String, assetId: String?, fromTimestamp _: UInt64) async throws -> Gemstone.TransactionsResponse {
        try lock.withLock { assetId == nil ? walletTransactionsResponse : assetTransactionsResponse }.json()
    }

    public func getAssetsList(walletId _: String, fromTimestamp _: UInt64) async throws -> [String] {
        assetsList.map(\.identifier)
    }

    public func setWalletTransactionsResponse(_ response: Primitives.TransactionsResponse) {
        lock.withLock { walletTransactionsResponse = response }
    }

    public func setAssetTransactionsResponse(_ response: Primitives.TransactionsResponse) {
        lock.withLock { assetTransactionsResponse = response }
    }
}

public actor GemWalletConfigurationServiceMock: GemWalletConfigurationServiceProtocol {
    private let result: Primitives.WalletConfigurationResult
    public private(set) var walletIds: [String] = []

    public init(
        result: Primitives.WalletConfigurationResult = WalletConfigurationResult(
            walletId: .multicoin(address: "mock"),
            configuration: WalletConfiguration(multiSignatureAccounts: []),
        ),
    ) {
        self.result = result
    }

    public func getConfiguration(walletId: String) async throws -> Gemstone.WalletConfigurationResult {
        walletIds.append(walletId)
        return try result.json()
    }
}

public final class GemSupportServiceMock: GemSupportServiceProtocol, @unchecked Sendable {
    private let messages: [Primitives.SupportMessage]

    public init(messages: [Primitives.SupportMessage] = []) {
        self.messages = messages
    }

    public func getMessages(fromTimestamp _: UInt64) async throws -> [Gemstone.SupportMessage] {
        try messages.map { try $0.json() }
    }

    public func sendMessage(input: Gemstone.SupportMessageInput) async throws -> Gemstone.SupportMessage {
        let content = try Primitives.SupportMessageInput(input).content
        return try Primitives.SupportMessage(id: UUID().uuidString, content: content, sender: .user, status: .sent, createdAt: .now, images: []).json()
    }

    public func sendImage(image _: Data, fileName _: String, mimeType _: String) async throws -> Gemstone.SupportMessage {
        try Primitives.SupportMessage(id: UUID().uuidString, content: "", sender: .user, status: .sent, createdAt: .now, images: []).json()
    }
}

public final class GemRewardsServiceMock: GemRewardsServiceProtocol, @unchecked Sendable {
    private let rewards: Primitives.Rewards?
    private let redemption: Primitives.RedemptionResult?

    public init(rewards: Primitives.Rewards? = nil, redemption: Primitives.RedemptionResult? = nil) {
        self.rewards = rewards
        self.redemption = redemption
    }

    public func getRewards(walletId _: String) async throws -> Gemstone.Rewards {
        guard let rewards else { throw AnyError("not stubbed") }
        return try rewards.json()
    }

    public func createReferral(walletId _: String, auth _: Gemstone.AuthPayload, code _: String) async throws -> Gemstone.Rewards {
        guard let rewards else { throw AnyError("not stubbed") }
        return try rewards.json()
    }

    public func useReferralCode(walletId _: String, auth _: Gemstone.AuthPayload, code _: String) async throws {}

    public func redeem(walletId _: String, auth _: Gemstone.AuthPayload, redemptionId _: String) async throws -> Gemstone.RedemptionResult {
        guard let redemption else { throw AnyError("not stubbed") }
        return try redemption.json()
    }
}

public final class GemNotificationServiceMock: GemNotificationServiceProtocol, @unchecked Sendable {
    private let notifications: [Primitives.InAppNotification]

    public init(notifications: [Primitives.InAppNotification] = []) {
        self.notifications = notifications
    }

    public func getNotifications(fromTimestamp _: UInt64) async throws -> [Gemstone.InAppNotification] {
        try notifications.map { try $0.json() }
    }

    public func markRead() async throws {}
}

public final class GemFiatServiceMock: GemFiatServiceProtocol, @unchecked Sendable {
    private let quotes: [Primitives.FiatQuote]
    private let transactions: [Primitives.FiatTransactionData]

    public init(quotes: [Primitives.FiatQuote] = [], transactions: [Primitives.FiatTransactionData] = []) {
        self.quotes = quotes
        self.transactions = transactions
    }

    public func getQuotes(walletId _: String, quoteType _: Gemstone.FiatQuoteType, assetId _: String, amount _: Double, currency _: String) async throws -> [Gemstone.FiatQuote] {
        try quotes.map { try $0.json() }
    }

    public func getQuoteUrl(walletId _: String, quoteId _: String) async throws -> Gemstone.FiatQuoteUrl {
        throw AnyError("not stubbed")
    }

    public func getTransactions(walletId _: String) async throws -> [Gemstone.FiatTransactionData] {
        try transactions.map { try $0.json() }
    }
}

public final class GemNameServiceMock: GemNameServiceProtocol, @unchecked Sendable {
    private let addressNames: [Primitives.AddressName]
    private let nameRecord: Primitives.NameRecord?
    private let error: Error?

    public init(addressNames: [Primitives.AddressName] = [], nameRecord: Primitives.NameRecord? = nil, error: Error? = nil) {
        self.addressNames = addressNames
        self.nameRecord = nameRecord
        self.error = error
    }

    public func resolve(name _: String, chain _: String) async throws -> Gemstone.NameRecord? {
        if let error { throw error }
        return try nameRecord?.json()
    }

    public func getAddressNames(requests: [Gemstone.ChainAddress]) async throws -> [Gemstone.AddressName] {
        if let error { throw error }
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
}

public final class GemAuthServiceMock: GemAuthServiceProtocol, @unchecked Sendable {
    private let nonce: Primitives.AuthNonce

    public init(nonce: Primitives.AuthNonce = AuthNonce(nonce: "nonce", timestamp: 0)) {
        self.nonce = nonce
    }

    public func getNonce() async throws -> Gemstone.AuthNonce {
        try nonce.json()
    }
}
