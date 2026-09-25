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

    public func perpetualLeverageSelection(maxLeverage: UInt8) -> GemLeverageSelection? {
        let options = stride(from: UInt8(1), through: maxLeverage, by: 1).map { GemPickerOption(value: $0, label: .text(text: "\($0)x")) }
        return options.first { $0.value == min(5, maxLeverage) }.map { GemLeverageSelection(options: options, selected: $0) }
    }

    public func transferData(asset: Gemstone.Asset, request: GemAmountRequest, value: Gemstone.GemBigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        if case .earn = request {
            throw AnyError("not stubbed")
        }
        return try await builder.transferData(asset: asset, request: request, value: value, useMaxAmount: useMaxAmount)
    }

    public var perpetualAutocloseValue: @Sendable (UInt8) -> GemPerpetualAutoclose = { _ in GemPerpetualAutoclose(takeProfit: nil, stopLoss: nil) }

    public func perpetualAutoclose(action _: GemPerpetualPositionAction, leverage: UInt8, decimalSeparator _: String) -> GemPerpetualAutoclose {
        perpetualAutocloseValue(leverage)
    }

    public func perpetualAutocloseRow(draft: GemAutocloseDraft, decimalSeparator: String) -> GemListRow {
        builder.perpetualAutocloseRow(draft: draft, decimalSeparator: decimalSeparator)
    }
}

public final class GemFiatQuoteServiceMock: GemFiatQuoteServiceProtocol, @unchecked Sendable {
    private let quotes: [Gemstone.FiatQuote]

    public init(quotes: [Gemstone.FiatQuote] = []) {
        self.quotes = quotes
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

    public func refreshTransactions(hasTransactions _: Bool) async -> GemLoadState {
        .data
    }

    public func quotes(quoteType _: Gemstone.FiatQuoteType, assetId _: Gemstone.AssetId, amount _: Double) async throws -> [Gemstone.FiatQuote] {
        quotes
    }

    public func quoteUrl(assetId _: Gemstone.AssetId, quoteId _: String) async throws -> Gemstone.FiatQuoteUrl {
        throw AnyError("not stubbed")
    }
}

public extension GemPaymentService {
    static func mock() -> GemPaymentService {
        GemPaymentService(provider: StubAlienProvider(), assets: .mock())
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

    public func nameInputStep(state: GemNameRecordState, name: String, chain: Gemstone.Chain?) -> GemNameInputStep {
        guard !name.isEmpty, let chain else {
            return .reset
        }
        switch state {
        case let .loading(loading, loadingChain) where loading == name && loadingChain == chain: return .unchanged
        case let .complete(record) where record.name == name && record.chain == chain: return .unchanged
        default: break
        }
        guard name.split(separator: ".").count >= 2 else {
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
    private let claimable: Bool
    private let actions: [Gemstone.GemDelegationAction]
    private let validators: [Gemstone.DelegationValidator]
    private let infoRows: [GemListRow]
    private let freezes: Bool
    private let wholeAmounts: Bool
    private let claimRewardsDestination: GemStakeDestination?
    private let refreshState: GemLoadState

    public init(
        claimable: Bool = false,
        actions: [Gemstone.GemDelegationAction] = [],
        validators: [Gemstone.DelegationValidator] = [],
        infoRows: [GemListRow] = [],
        freezes: Bool = false,
        wholeAmounts: Bool = false,
        claimRewardsDestination: GemStakeDestination? = nil,
        refreshState: GemLoadState = .data,
    ) {
        self.claimable = claimable
        self.actions = actions
        self.validators = validators
        self.infoRows = infoRows
        self.freezes = freezes
        self.wholeAmounts = wholeAmounts
        self.claimRewardsDestination = claimRewardsDestination
        self.refreshState = refreshState
    }

    public func stakeValidatorSelection(chain _: Gemstone.Chain, input _: GemStakeAmountInput) -> GemStakeValidatorSelection {
        let options = validators.map { Gemstone.GemValidatorRow.mock(validator: $0) }
        return GemStakeValidatorSelection(options: options, recommended: [], validator: options.first, canSelect: true)
    }

    public func stakeViewState(input: GemStakeInput) -> GemStakeViewState {
        let actions = [
            GemStakeActionItem(
                action: .stake,
                row: .action(title: .stake, value: nil, info: nil),
                tap: validators.isEmpty ? .disabled : .open(destination: .amount(input: .stake(validators: validators, validator: nil))),
            ),
            GemStakeActionItem(
                action: .claimRewards,
                row: .action(title: .claimRewards, value: nil, info: nil),
                tap: .open(destination: claimRewardsDestination ?? .amount(input: .rewards(delegations: input.delegations, validator: nil))),
            ),
        ]
        return GemStakeViewState(
            sections: [.manage, freezes ? .resources : nil, input.delegations.isEmpty ? nil : .delegations].compactMap(\.self),
            infoRows: infoRows,
            actions: actions,
            resourceRows: [],
            delegations: zip(input.delegations, Gemstone.delegationListRows(delegations: input.delegations, asset: input.asset, price: input.price, currency: input.currency)).map {
                GemStakeDelegationItem(delegation: $0, row: $1, destination: .details)
            },
            validators: validators,
            docsUrl: nil,
        )
    }

    public func delegationRows(delegation _: Gemstone.Delegation) -> [GemListRow] {
        []
    }

    public func delegationDetails(walletType: Gemstone.WalletType, delegation: Gemstone.Delegation, asset: Gemstone.Asset, price: Double?, currency: Gemstone.Currency) -> GemDelegationDetails {
        var details = Gemstone.delegationDetails(walletType: walletType, delegation: delegation, asset: asset, price: price, currency: currency)
        details.actions = actions
        if !claimable {
            details.claim = nil
        }
        return details
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

    public func getCurrency() -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func resourceOptions(chain _: Gemstone.Chain) -> [Gemstone.Resource] {
        [.bandwidth, .energy]
    }

    public func refresh(chain _: Gemstone.Chain, delegations _: [Gemstone.Delegation]) async -> GemLoadState {
        refreshState
    }

    public func syncEarn(assetId _: Gemstone.AssetId) async throws {}

    public func refreshEarn(assetId _: Gemstone.AssetId, hasRows _: Bool) async -> GemLoadState {
        refreshState
    }

    public func earnView(input: GemEarnInput) -> GemEarnView {
        let positions = input.delegations.filter { BigInt($0.base.balance) > 0 }
        return GemEarnView(
            aprRow: .text(title: .stakeApr, value: ""),
            providers: input.providers,
            depositProvider: input.walletType == .view ? nil : input.providers.first,
            positions: zip(positions, Gemstone.delegationListRows(delegations: positions, asset: input.asset, price: input.price, currency: input.currency)).map {
                GemStakeDelegationItem(delegation: $0, row: $1, destination: .details)
            },
        )
    }
}

public final class GemTransactionStateServiceMock: GemTransactionStateServiceProtocol, @unchecked Sendable {
    private var status: (any GemTransactionStatusService)?

    public init() {}

    public func setStatus(status: any GemTransactionStatusService) {
        self.status = status
    }

    public func trackPending() async throws {}

    public func track(walletId _: Gemstone.WalletId, transactions _: [Gemstone.Transaction]) async throws {}

    public func stopTracking() {}
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
