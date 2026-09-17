// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import struct Gemstone.GemPriceAlertSession
import enum Gemstone.GemNameInputStep

public final class GemAmountServiceMock: GemAmountServiceProtocol, @unchecked Sendable {
    private let builder: any GemAmountServiceProtocol

    public init(builder: any GemAmountServiceProtocol) {
        self.builder = builder
    }

    public func getCurrency() -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func stakeTransferData(asset: Gemstone.Asset, stakeType: Gemstone.StakeType, value: Gemstone.GemBigInt, useMaxAmount: Bool) -> GemTransferData {
        builder.stakeTransferData(asset: asset, stakeType: stakeType, value: value, useMaxAmount: useMaxAmount)
    }

    public func perpetualTransferData(action: GemPerpetualPositionAction, value: Gemstone.GemBigInt, useMaxAmount: Bool, leverage: UInt8, takeProfit: Double?, stopLoss: Double?) -> GemTransferData {
        builder.perpetualTransferData(action: action, value: value, useMaxAmount: useMaxAmount, leverage: leverage, takeProfit: takeProfit, stopLoss: stopLoss)
    }

    public func earnTransferData(asset _: Gemstone.Asset, earnType _: Gemstone.EarnType, value _: Gemstone.GemBigInt, useMaxAmount _: Bool) async throws -> GemTransferData {
        throw AnyError("not stubbed")
    }


    public func perpetualLeverage(maxLeverage: UInt8) -> UInt8 {
        min(5, maxLeverage)
    }

    public func perpetualAmountType(action: GemPerpetualPositionAction, leverage: UInt8) -> GemAmountType {
        builder.perpetualAmountType(action: action, leverage: leverage)
    }

    public func stakeValidatorSelection(chain: Gemstone.Chain, input: GemStakeAmountInput) -> GemStakeValidatorSelection {
        builder.stakeValidatorSelection(chain: chain, input: input)
    }

    public func earnAmountType(earnType: Gemstone.EarnType) -> GemAmountType {
        builder.earnAmountType(earnType: earnType)
    }

    public func validatorRow(validator: Gemstone.DelegationValidator) -> Gemstone.GemValidatorRow {
        builder.validatorRow(validator: validator)
    }

    public func validatorRows(validators: [Gemstone.DelegationValidator]) -> [Gemstone.GemValidatorRow] {
        validators.map { builder.validatorRow(validator: $0) }
    }

    public func transferData(asset: Gemstone.Asset, transfer: GemAmountTransfer, value: Gemstone.GemBigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        try await builder.transferData(asset: asset, transfer: transfer, value: value, useMaxAmount: useMaxAmount)
    }

    public func perpetualAutoclose(price _: Double, direction _: Gemstone.PerpetualDirection, leverage _: UInt8) -> GemPerpetualAutoclose {
        GemPerpetualAutoclose(takeProfit: nil, stopLoss: nil)
    }
}

public final class GemFiatQuoteServiceMock: GemFiatQuoteServiceProtocol, @unchecked Sendable {
    private let quotes: [Gemstone.FiatQuote]

    public init(quotes: [Gemstone.FiatQuote] = []) {
        self.quotes = quotes
    }

    public func getCurrency() -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func suggestedAmounts() -> [Int32] {
        [100, 250]
    }

    private func defaultAmount(quoteType: Gemstone.FiatQuoteType) -> UInt32 {
        quoteType.toPrimitives() == .sell ? 100 : 50
    }

    public func newSession(quoteType: Gemstone.FiatQuoteType, amount: UInt32?) -> GemFiatSession {
        let operation = { (type: Gemstone.FiatQuoteType) -> GemFiatOperation in
            let value = (type == quoteType ? amount : nil) ?? self.defaultAmount(quoteType: type)
            return GemFiatOperation(quoteType: type, amount: String(value), quotes: [], selectedProvider: nil, phase: .loading(amount: Double(value)))
        }
        return GemFiatSession(quoteType: quoteType, buy: operation(.buy), sell: operation(.sell), available: 0)
    }

    public func randomAmount() -> UInt32 {
        50
    }

    public func quoteDebounceMilliseconds() -> UInt64 {
        250
    }

    public func quoteRefreshIntervalMilliseconds() -> UInt64 {
        300_000
    }

    public func syncTransactions() async throws {}

    public func quotes(quoteType _: Gemstone.FiatQuoteType, assetId _: Gemstone.AssetId, amount _: Double) async throws -> [Gemstone.FiatQuote] {
        quotes
    }

    public func quoteUrl(assetId _: Gemstone.AssetId, quoteId _: String) async throws -> Gemstone.FiatQuoteUrl {
        throw AnyError("not stubbed")
    }
}

public extension GemNameService {
    static func mock() -> GemNameService {
        GemNameService(
            api: GemDeviceApiClient(
                provider: StubAlienProvider(),
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

    public func getNameRecord(name: String, chain _: String) async throws -> GemNameRecordState {
        requestedNames.append(name)
        if let error { throw error }
        return nameRecord.map { .complete(record: $0.toGem()) } ?? .error
    }

    public func isNameSupported(name: String) -> Bool {
        name.split(separator: ".").count >= 2
    }

    public func nameInputStep(state: GemNameRecordState, name: String, hasChain: Bool) -> GemNameInputStep {
        if name.isEmpty {
            return .reset
        }
        switch state {
        case let .loading(loading) where loading == name: return .unchanged
        case let .complete(record) where record.name == name: return .unchanged
        default: break
        }
        guard hasChain, isNameSupported(name: name) else {
            return .reset
        }
        return .resolve(name: name, debounceMilliseconds: 0)
    }

    public func resolvedState(state: GemNameRecordState, name: String, resolved: GemNameRecordState) -> GemNameRecordState {
        state == .loading(name: name) ? resolved : state
    }

    public func validateRecipient(chain: Gemstone.Chain, input: String, state: GemNameRecordState) -> GemRecipientValidation {
        rules.validateRecipient(chain: chain, input: input, state: state)
    }

    public func recipient(chain: Gemstone.Chain, input: String, state: GemNameRecordState, memo: String?, references: [String]) throws -> GemRecipient {
        try rules.recipient(chain: chain, input: input, state: state, memo: memo, references: references)
    }
}

public final class GemStakeServiceMock: GemStakeServiceProtocol, @unchecked Sendable {
    private let rewardsShown: Bool
    private let claimable: Bool
    private let explorerAddress: String?
    private let actions: [Gemstone.GemDelegationAction]
    private let validators: [Gemstone.DelegationValidator]
    private let lockTime: UInt64
    private let minStake: Gemstone.GemBigInt
    private let freezes: Bool
    private let wholeAmounts: Bool

    public init(
        rewardsShown: Bool = false,
        claimable: Bool = false,
        explorerAddress: String? = nil,
        actions: [Gemstone.GemDelegationAction] = [],
        validators: [Gemstone.DelegationValidator] = [],
        lockTime: UInt64 = 0,
        minStake: Gemstone.GemBigInt = 0,
        freezes: Bool = false,
        wholeAmounts: Bool = false,
    ) {
        self.rewardsShown = rewardsShown
        self.claimable = claimable
        self.explorerAddress = explorerAddress
        self.actions = actions
        self.validators = validators
        self.lockTime = lockTime
        self.minStake = minStake
        self.freezes = freezes
        self.wholeAmounts = wholeAmounts
    }

    public func sortedDelegations(delegations: [Gemstone.Delegation]) -> [Gemstone.Delegation] {
        delegations
    }

    public func lockTimeSeconds(chain _: Gemstone.Chain) -> UInt64 {
        lockTime
    }

    public func minStakeAmount(chain _: Gemstone.Chain) -> Gemstone.GemBigInt {
        minStake
    }

    public func earnApr(providers: [Gemstone.DelegationValidator], assetApr: Double?) -> Double {
        providers.first.map(\.apr).flatMap { $0 > 0 ? $0 : nil } ?? assetApr ?? 0
    }

    public func stakeSections(chain _: Gemstone.Chain, hasActions: Bool, hasDelegations: Bool) -> [Gemstone.GemStakeSection] {
        [hasActions ? .manage : nil, freezes ? .resources : nil, hasDelegations ? .delegations : nil].compactMap { $0 }
    }

    public func stakeInfoRows(chain _: Gemstone.Chain, stakingApr: Double?) -> [Gemstone.GemStakeInfoRow] {
        [stakingApr.flatMap { $0 != 0 ? .apr : nil }, lockTime > 0 ? .lockTime : nil, minStake != 0 ? .minimumAmount : nil].compactMap { $0 }
    }

    public func delegationRows(delegation: Gemstone.Delegation) -> [Gemstone.GemDelegationRow] {
        [.provider, delegation.validator.apr != 0 ? .apr : nil, .status, rewardsShown ? .rewards : nil].compactMap { $0 }
    }


    public func stakeTransferData(asset: Gemstone.Asset, stakeType: Gemstone.StakeType, value: Gemstone.GemBigInt, useMaxAmount: Bool) -> GemTransferData {
        GemTransferData(inputType: .stake(asset: asset, stakeType: stakeType), recipient: GemRecipient(address: ""), value: value, useMaxAmount: useMaxAmount)
    }

    public func delegationDestination(walletType _: Gemstone.WalletType, asset _: Gemstone.Asset, delegation _: Gemstone.Delegation) -> GemDelegationDestination {
        .details
    }

    public func delegationActionDestination(
        asset _: Gemstone.Asset,
        delegation _: Gemstone.Delegation,
        action _: Gemstone.GemDelegationAction,
        validators _: [Gemstone.DelegationValidator],
    ) -> GemDelegationDestination {
        .details
    }

    public func delegationActions(walletType _: Gemstone.WalletType, delegation _: Gemstone.Delegation) -> [Gemstone.GemDelegationAction] {
        actions
    }

    public func canClaimDelegationRewards(walletType _: Gemstone.WalletType, delegation _: Gemstone.Delegation) -> Bool {
        claimable
    }

    public func getCurrency() -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func validatorRow(validator: Gemstone.DelegationValidator) -> Gemstone.GemValidatorRow {
        .mock(validator: validator)
    }

    public func validatorRows(validators: [Gemstone.DelegationValidator]) -> [Gemstone.GemValidatorRow] {
        validators.map { validatorRow(validator: $0) }
    }

    public func validatorUrl(validator _: Gemstone.DelegationValidator) -> Gemstone.BlockExplorerLink? {
        explorerAddress.map { Gemstone.BlockExplorerLink(name: "MockExplorer", link: "https://explorer.mock/validator/\($0)") }
    }

    public func showsRewards(delegation _: Gemstone.DelegationBase) -> Bool {
        rewardsShown
    }

    public func stakeActions(walletType _: Gemstone.WalletType, chain _: Gemstone.Chain, hasValidators: Bool, balance _: GemAssetBalance, delegations _: [Gemstone.Delegation]) -> [GemStakeActionItem] {
        [GemStakeActionItem(action: .stake, isEnabled: hasValidators, requiresFrozenBalance: false)]
    }

    public func claimRewards(chain _: Gemstone.Chain, delegations: [Gemstone.Delegation]) -> GemClaimRewards {
        GemClaimRewards(value: 0, destination: .amount(delegations: delegations))
    }

    public func selectableValidators(validators _: [Gemstone.DelegationValidator]) -> [Gemstone.DelegationValidator] {
        validators
    }

    public func sync(chain _: Gemstone.Chain) async throws {}

    public func syncEarn(assetId _: Gemstone.AssetId) async throws {}
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

public extension Gemstone.GemFeeAsset {
    static func mock(
        asset: Primitives.Asset,
        balance: Gemstone.GemAssetBalance? = nil,
        price: Gemstone.AssetPrice? = nil,
    ) -> Gemstone.GemFeeAsset {
        Gemstone.GemFeeAsset(
            asset: asset.toGem(),
            balance: balance ?? .mock(assetId: asset.id.identifier),
            price: price,
        )
    }
}

public final class GemReceiveServiceMock: GemReceiveServiceProtocol, @unchecked Sendable {
    public var networkAssetIdsValue: [Gemstone.AssetId] = []
    public var warningsValue: [GemReceiveWarning] = []
    public var assetResult: Result<Gemstone.Asset, Error> = .success(Primitives.Asset.mock().toGem())
    public var enableAssetError: Error?
    public var syncedNetworkAssetIdsResult: Result<[Gemstone.AssetId], Error> = .success([])

    public private(set) var enabledAssetIds: [Gemstone.AssetId] = []
    public private(set) var syncedAssetIds: [Gemstone.AssetId] = []
    public private(set) var requestedAssetIds: [Gemstone.AssetId] = []

    public init() {}

    public func asset(assetId: Gemstone.AssetId) async throws -> Gemstone.Asset {
        requestedAssetIds.append(assetId)
        return try assetResult.get()
    }

    public func enableAsset(walletId _: Gemstone.WalletId, assetId: Gemstone.AssetId) async throws {
        enabledAssetIds.append(assetId)
        if let enableAssetError { throw enableAssetError }
    }

    public func networkAssetIds(assetId: Gemstone.AssetId, associations _: [Gemstone.AssetId], wallet _: Gemstone.Wallet) -> [Gemstone.AssetId] {
        networkAssetIdsValue.isEmpty ? [assetId] : networkAssetIdsValue
    }

    public func syncNetworkAssetIds(assetId: Gemstone.AssetId, wallet _: Gemstone.Wallet) async throws -> [Gemstone.AssetId] {
        syncedAssetIds.append(assetId)
        return try syncedNetworkAssetIdsResult.get()
    }

    public func warnings(chain _: Gemstone.Chain) -> [GemReceiveWarning] { warningsValue }
}

public final class GemTransactionsServiceMock: GemTransactionsServiceProtocol, @unchecked Sendable {
    public var filterChainsValue: [Gemstone.Chain] = []
    public var syncError: Error?

    public private(set) var syncedAssetIds: [Gemstone.AssetId?] = []

    public init(filterChains: [Gemstone.Chain] = []) {
        filterChainsValue = filterChains
    }

    public func filterChains(wallet _: Gemstone.Wallet) -> [Gemstone.Chain] { filterChainsValue }

    public func getCurrency() -> Gemstone.Currency { Primitives.Currency.usd.toGem() }

    public func sync(assetId: Gemstone.AssetId?) async throws {
        syncedAssetIds.append(assetId)
        if let syncError { throw syncError }
    }
}
