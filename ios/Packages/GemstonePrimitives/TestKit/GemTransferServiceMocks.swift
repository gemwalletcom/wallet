// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import enum Gemstone.GemNameInputStep
import struct Gemstone.GemPriceAlertSession
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public final class GemAmountServiceMock: GemAmountServiceProtocol, @unchecked Sendable {
    private let builder: any GemAmountServiceProtocol

    public init(builder: any GemAmountServiceProtocol) {
        self.builder = builder
    }

    public func getCurrency() -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
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

    public func earnAmountType(earnType: Gemstone.EarnType) -> GemAmountType {
        builder.earnAmountType(earnType: earnType)
    }

    public func transferData(asset: Gemstone.Asset, transfer: GemAmountTransfer, value: Gemstone.GemBigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        try await builder.transferData(asset: asset, transfer: transfer, value: value, useMaxAmount: useMaxAmount)
    }

    public func perpetualAutoclose(price _: Double, direction _: Gemstone.PerpetualDirection, leverage _: UInt8) -> GemPerpetualAutoclose {
        GemPerpetualAutoclose(takeProfit: nil, stopLoss: nil)
    }

    public func perpetualAutocloseRow(takeProfit: Double?, stopLoss: Double?) -> GemListRow {
        builder.perpetualAutocloseRow(takeProfit: takeProfit, stopLoss: stopLoss)
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

    public func suggestedAmounts() -> [GemFiatSuggestedAmount] {
        [100, 250].map { GemFiatSuggestedAmount(amount: $0, value: .mock(value: Double($0), display: .number(precision: .fraction(min: 0, max: 0)), notation: .plain)) }
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
        if let error {
            throw error
        }
        return nameRecord.map { .complete(record: $0.toGem()) } ?? .error
    }

    public func isNameSupported(name: String) -> Bool {
        name.split(separator: ".").count >= 2
    }

    public func nameInputStep(state: GemNameRecordState, name: String, chain: Gemstone.Chain?) -> GemNameInputStep {
        guard !name.isEmpty, let chain else {
            return .reset
        }
        switch state {
        case let .loading(loading, loadingChain) where loading == name && loadingChain == chain: return .unchanged
        case let .complete(record) where record.name == name && record.chain == chain: return .unchanged
        default: break
        }
        guard isNameSupported(name: name) else {
            return .reset
        }
        return .resolve(name: name, debounceMilliseconds: 0)
    }

    public func resolvedState(state: GemNameRecordState, name: String, chain: Gemstone.Chain, resolved: GemNameRecordState) -> GemNameRecordState {
        state == .loading(name: name, chain: chain) ? resolved : state
    }

    public func validateRecipient(chain: Gemstone.Chain, input: String, state: GemNameRecordState) -> GemRecipientValidation {
        rules.validateRecipient(chain: chain, input: input, state: state)
    }
}

public final class GemStakeServiceMock: GemStakeServiceProtocol, @unchecked Sendable {
    private let rewardsShown: Bool
    private let claimable: Bool
    private let explorerAddress: String?
    private let actions: [Gemstone.GemDelegationAction]
    private let validators: [Gemstone.DelegationValidator]
    private let infoRows: [GemListRow]
    private let freezes: Bool
    private let wholeAmounts: Bool
    private let claimRewardsDestination: GemClaimRewardsDestination?
    private let refreshState: GemLoadState

    public init(
        rewardsShown: Bool = false,
        claimable: Bool = false,
        explorerAddress: String? = nil,
        actions: [Gemstone.GemDelegationAction] = [],
        validators: [Gemstone.DelegationValidator] = [],
        infoRows: [GemListRow] = [],
        freezes: Bool = false,
        wholeAmounts: Bool = false,
        claimRewardsDestination: GemClaimRewardsDestination? = nil,
        refreshState: GemLoadState = .data,
    ) {
        self.rewardsShown = rewardsShown
        self.claimable = claimable
        self.explorerAddress = explorerAddress
        self.actions = actions
        self.validators = validators
        self.infoRows = infoRows
        self.freezes = freezes
        self.wholeAmounts = wholeAmounts
        self.claimRewardsDestination = claimRewardsDestination
        self.refreshState = refreshState
    }

    public func sortedDelegations(delegations: [Gemstone.Delegation]) -> [Gemstone.Delegation] {
        delegations
    }

    public func stakeValidatorSelection(chain _: Gemstone.Chain, input _: GemStakeAmountInput) -> GemStakeValidatorSelection {
        let options = validators.map { Gemstone.GemValidatorRow.mock(validator: $0) }
        return GemStakeValidatorSelection(options: options, recommended: [], validator: options.first, canSelect: true)
    }

    public func positions(delegations: [Gemstone.Delegation]) -> [Gemstone.Delegation] {
        delegations.filter { BigInt($0.base.balance) > 0 }
    }

    public func earnAprRow(providers _: [Gemstone.DelegationValidator], assetApr _: Double?) -> GemListRow {
        .text(title: .stakeApr, value: "")
    }

    public func stakeSections(chain _: Gemstone.Chain, hasActions: Bool, hasDelegations: Bool) -> [Gemstone.GemStakeSection] {
        [hasActions ? .manage : nil, freezes ? .resources : nil, hasDelegations ? .delegations : nil].compactMap(\.self)
    }

    public func stakeInfoRows(asset _: Gemstone.Asset, stakingApr _: Double?) -> [GemListRow] {
        infoRows
    }

    public func delegationRows(delegation _: Gemstone.Delegation) -> [GemListRow] {
        []
    }

    public func delegationDetails(delegation: Gemstone.Delegation, asset: Gemstone.Asset, price: Double?, currency: Gemstone.Currency) -> GemDelegationDetails {
        Gemstone.delegationDetails(delegation: delegation, asset: asset, price: price, currency: currency)
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

    public func validatorRows(validators: [Gemstone.DelegationValidator]) -> [Gemstone.GemValidatorRow] {
        validators.map { Gemstone.validatorRow(validator: $0) }
    }

    public func validatorUrl(validator _: Gemstone.DelegationValidator) -> Gemstone.BlockExplorerLink? {
        explorerAddress.map { Gemstone.BlockExplorerLink(name: "MockExplorer", link: "https://explorer.mock/validator/\($0)") }
    }

    public func showsRewards(delegation _: Gemstone.DelegationBase) -> Bool {
        rewardsShown
    }

    public func resourceOptions(chain _: Gemstone.Chain) -> [Gemstone.Resource] {
        [.bandwidth, .energy]
    }

    public func stakeActions(walletType _: Gemstone.WalletType, chain _: Gemstone.Chain, validators: [Gemstone.DelegationValidator], balance _: GemAssetBalance, delegations _: [Gemstone.Delegation]) -> [GemStakeActionItem] {
        [
            GemStakeActionItem(
                action: .stake,
                isEnabled: validators.isEmpty == false,
                requiresFrozenBalance: false,
                value: nil,
                destination: .amount(input: .stake(validators: validators, validator: nil)),
            ),
        ]
    }

    public func claimRewards(chain _: Gemstone.Chain, delegations: [Gemstone.Delegation]) -> GemClaimRewards {
        GemClaimRewards(destination: claimRewardsDestination ?? .amount(delegations: delegations))
    }

    public func selectableValidators(validators _: [Gemstone.DelegationValidator]) -> [Gemstone.DelegationValidator] {
        validators
    }

    public func refresh(chain _: Gemstone.Chain, delegations _: [Gemstone.Delegation]) async -> GemLoadState {
        refreshState
    }

    public func syncEarn(assetId _: Gemstone.AssetId) async throws {}

    public func refreshEarn(assetId _: Gemstone.AssetId, hasRows _: Bool) async -> GemLoadState {
        refreshState
    }

    public func earnActions(walletType: Gemstone.WalletType, providers: [Gemstone.DelegationValidator]) -> GemEarnActions {
        GemEarnActions(depositProvider: walletType == .view ? nil : providers.first)
    }
}

public final class GemTransactionStateServiceMock: GemTransactionStateServiceProtocol, @unchecked Sendable {
    private let store: (any GemTransactionStateStore)?
    private let notificationAsset: Gemstone.Asset?
    private var status: (any GemTransactionStatusService)?

    public init(store: (any GemTransactionStateStore)? = nil, notificationAsset: Gemstone.Asset? = nil) {
        self.store = store
        self.notificationAsset = notificationAsset
    }

    public func setStatus(status: any GemTransactionStatusService) {
        self.status = status
    }

    public func trackPending() async throws {}

    public func track(walletId _: Gemstone.WalletId, transactions _: [Gemstone.Transaction]) async throws {}

    public func stopTracking() {}

    public func addNotificationTransaction(wallet _: Gemstone.Wallet, assetId _: Gemstone.AssetId, transaction: Gemstone.Transaction) async throws -> Gemstone.Asset? {
        if let store {
            try await store.addTransactions(walletId: "", transactions: [transaction])
        }
        status?.track(walletId: "", transactions: [transaction])
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
    public var networksValue: GemReceiveNetworks?
    public var warningsValue: [GemReceiveWarning] = []
    public var assetResult: Result<Gemstone.Asset, Error> = .success(Primitives.Asset.mock().toGem())
    public var assetsById: [Gemstone.AssetId: Gemstone.Asset] = [:]
    public var enableAssetError: Error?

    public private(set) var enabledAssetIds: [Gemstone.AssetId] = []
    public private(set) var requestedAssetIds: [Gemstone.AssetId] = []

    public init() {}

    public func asset(assetId: Gemstone.AssetId) async throws -> Gemstone.Asset {
        requestedAssetIds.append(assetId)
        if let asset = assetsById[assetId] {
            return asset
        }
        return try assetResult.get()
    }

    public func enableAsset(walletId _: Gemstone.WalletId, assetId: Gemstone.AssetId) async throws {
        enabledAssetIds.append(assetId)
        if let enableAssetError {
            throw enableAssetError
        }
    }

    public func networks(assetId: Gemstone.AssetId, associations _: [Gemstone.AssetId], wallet _: Gemstone.Wallet) -> GemReceiveNetworks {
        networksValue ?? GemReceiveNetworks(assetIds: [assetId], showsSelector: false)
    }

    public func warnings(chain _: Gemstone.Chain) -> [GemReceiveWarning] {
        warningsValue
    }
}

public final class GemTransactionsServiceMock: GemTransactionsServiceProtocol, @unchecked Sendable {
    public var filterChainsValue: [Gemstone.Chain] = []
    public var refreshState: GemLoadState = .data

    public private(set) var syncedAssetIds: [Gemstone.AssetId?] = []

    public init(filterChains: [Gemstone.Chain] = []) {
        filterChainsValue = filterChains
    }

    public func filterChains(wallet _: Gemstone.Wallet) -> [Gemstone.Chain] {
        filterChainsValue
    }

    public func refresh(assetId: Gemstone.AssetId?, hasTransactions _: Bool) async -> GemLoadState {
        syncedAssetIds.append(assetId)
        return refreshState
    }
}
