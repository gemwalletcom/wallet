// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
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

    public func transferData(asset: Gemstone.Asset, request: GemAmountRequest, value: Gemstone.GemBigInt, useMaxAmount: Bool) async throws -> GemTransferData {
        if case .earn = request {
            throw AnyError("not stubbed")
        }
        return try await builder.transferData(asset: asset, request: request, value: value, useMaxAmount: useMaxAmount)
    }

    public func extras(request: GemAmountRequest, asset: Gemstone.Asset) -> GemAmountExtras {
        builder.extras(request: request, asset: asset)
    }
}

public final class GemFiatQuoteServiceMock: GemFiatQuoteServiceProtocol, @unchecked Sendable {
    public var unavailableTopicValue: GemInfoTopic?
    public var refreshTransactionsState: GemLoadState = .data
    public private(set) var quoteUrlRequests: [String] = []

    public func unavailableTopic(quoteType _: Gemstone.FiatQuoteType) -> GemInfoTopic? {
        unavailableTopicValue
    }

    private let quotes: [Gemstone.FiatQuote]

    public init(quotes: [Gemstone.FiatQuote] = []) {
        self.quotes = quotes
    }

    public func suggestedAmounts() -> [GemFiatSuggestedAmount] {
        [100, 250].map { GemFiatSuggestedAmount(amount: $0, value: .mock(value: Double($0), unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 0, max: 0)), notation: .plain, tone: .plain, rounding: .toNearest)) }
    }

    public static let defaultBuyAmount: UInt32 = 20
    public static let defaultSellAmount: UInt32 = 40

    private func defaultAmount(quoteType: Gemstone.FiatQuoteType) -> UInt32 {
        quoteType.toPrimitives() == .sell ? Self.defaultSellAmount : Self.defaultBuyAmount
    }

    public func newSession(quoteType: Gemstone.FiatQuoteType, amount: UInt32?, format: GemNumberFormat) -> GemFiatSession {
        let operation = { (type: Gemstone.FiatQuoteType) -> GemFiatOperation in
            let value = (type == quoteType ? amount : nil) ?? self.defaultAmount(quoteType: type)
            return GemFiatOperation(quoteType: type, amount: String(value), quotes: [], selectedProvider: nil, phase: .loading(amount: Double(value)))
        }
        return GemFiatSession(quoteType: quoteType, buy: operation(.buy), sell: operation(.sell), available: 0, format: format)
    }

    public func refreshTransactions() async -> GemLoadState {
        refreshTransactionsState
    }

    public func quotes(request: GemFiatQuoteRequest, assetId _: Gemstone.AssetId) async -> GemFiatQuotesResult {
        GemFiatQuotesResult(request: request, quotes: quotes, error: nil)
    }

    public func quoteUrl(assetId _: Gemstone.AssetId, quoteId: String) async throws -> Gemstone.FiatQuoteUrl {
        quoteUrlRequests.append(quoteId)
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
    private let nameRecord: Gemstone.NameRecord?
    private let error: Error?
    public private(set) var requestedNames: [String] = []

    public init(nameRecord: Gemstone.NameRecord? = nil, error: Error? = nil) {
        self.nameRecord = nameRecord
        self.error = error
    }

    public func getNameRecord(name: String, chain _: String) async throws -> GemNameRecordState {
        requestedNames.append(name)
        if let error {
            throw error
        }
        return nameRecord.map { .complete(record: $0) } ?? .error
    }
}

public final class GemStakeServiceMock: GemStakeServiceProtocol, @unchecked Sendable {
    private let details: GemDelegationDetails
    private let validators: [Gemstone.DelegationValidator]
    private let infoRows: [GemListRow]
    private let claimRewardsDestination: GemStakeDestination?
    private let refreshState: GemLoadState

    public init(
        details: GemDelegationDetails = .mock(),
        validators: [Gemstone.DelegationValidator] = [],
        infoRows: [GemListRow] = [],
        claimRewardsDestination: GemStakeDestination? = nil,
        refreshState: GemLoadState = .data,
    ) {
        self.details = details
        self.validators = validators
        self.infoRows = infoRows
        self.claimRewardsDestination = claimRewardsDestination
        self.refreshState = refreshState
    }

    public func stakeValidatorOptions(chain _: Gemstone.Chain, input: GemStakeAmountInput, validators: [Gemstone.DelegationValidator]) -> GemGroupList {
        GemGroupList(
            groups: [
                GemGroup(
                    id: "Active",
                    title: .sectionTitle(title: .active),
                    footer: .none,
                    items: validators.map {
                        .validator(
                            row: Gemstone.GemValidatorRow.mock(validator: $0, name: $0.name, imageUrl: "https://assets.gemwallet.com/validator.png", placeholder: String($0.name.prefix(1)), apr: .apr(value: nil)),
                            isSelected: $0.id == input.selectedValidatorId,
                        )
                    },
                ),
            ].filter(\.items.isNotEmpty),
            phase: validators.isEmpty ? .empty(state: emptyState(kind: .validators)) : .rows,
        )
    }

    public func stakeViewState(input: GemStakeInput) -> GemStakeViewState {
        let actions = [
            GemStakeActionItem(
                kind: .stake,
                row: .action(title: .stake, value: nil, info: nil),
                action: validators.first.map { .open(destination: .amount(input: .stake(validator: $0))) } ?? .disabled,
            ),
            GemStakeActionItem(
                kind: .claimRewards,
                row: .action(title: .claimRewards, value: nil, info: nil),
                action: (claimRewardsDestination ?? input.delegations.first.map { .amount(input: .rewards(delegations: input.delegations, validator: $0.validator)) })
                    .map { .open(destination: $0) } ?? .disabled,
            ),
        ]
        return GemStakeViewState(
            asset: Gemstone.assetText(asset: input.assetData.asset),
            sections: [.manage, .delegations],
            infoRows: infoRows,
            actions: actions,
            resourceRows: [],
            delegations: input.delegations.map(delegationItem),
            delegationsPhase: loadedPhase(state: input.state, hasRows: !input.delegations.isEmpty, kind: .stake),
            docsUrl: nil,
        )
    }

    public func delegationDetails(
        walletType _: Gemstone.WalletType,
        delegation _: Gemstone.Delegation,
        asset _: Gemstone.Asset,
        price _: Double?,
        currency _: Gemstone.Currency,
        validators _: [Gemstone.DelegationValidator],
    ) -> GemDelegationDetails {
        details
    }

    public func getCurrency() -> Gemstone.Currency {
        Primitives.Currency.usd.toGem()
    }

    public func refresh(chain _: Gemstone.Chain) async -> GemLoadState {
        refreshState
    }

    public func refreshEarn(assetId _: Gemstone.AssetId) async -> GemLoadState {
        refreshState
    }

    public func earnView(input: GemEarnInput) -> GemEarnView {
        GemEarnView(
            asset: Gemstone.assetText(asset: input.asset),
            rateRow: .text(title: .stakeApr, value: ""),
            sections: [.manage, .positions],
            depositRow: .action(title: .deposit, value: nil, info: nil),
            depositProvider: input.providers.first,
            positions: input.delegations.map(delegationItem),
            positionsPhase: loadedPhase(state: input.state, hasRows: !input.delegations.isEmpty, kind: .earn),
        )
    }

    private func delegationItem(_ delegation: Gemstone.Delegation) -> GemStakeDelegationItem {
        GemStakeDelegationItem(id: delegation.base.delegationId, delegation: delegation, row: .mock(), destination: .details)
    }
}

private func loadedPhase(state: GemLoadState, hasRows: Bool, kind: GemEmptyStateKind) -> GemListPhase? {
    switch (hasRows, state) {
    case (true, _): .rows
    case (false, .loading): nil
    case let (false, .error(error)): .error(error: error)
    case (false, .data), (false, .noData): .empty(state: emptyState(kind: kind))
    }
}

public final class GemTransactionStateServiceMock: GemTransactionStateServiceProtocol, @unchecked Sendable {
    public init() {}

    public func setStatus(status _: any GemTransactionStatusService) {}

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

public final class GemReceiveServiceMock: GemReceiveServiceProtocol, @unchecked Sendable {
    var updateError: Error?
    public var warningsValue: [GemReceiveWarning] = []
    public var assetResult: Result<Gemstone.Asset, Error> = .success(Primitives.Asset.mock().toGem())
    public var assetsById: [Gemstone.AssetId: Gemstone.Asset] = [:]
    public var heldAssetId: Gemstone.AssetId?
    public var enableAssetError: Error?
    private let (heldAsset, releaseHeld) = AsyncStream<Void>.makeStream()

    public private(set) var enabledAssetIds: [Gemstone.AssetId] = []
    public private(set) var requestedAssetIds: [Gemstone.AssetId] = []
    public private(set) var updatedAssetIds: [Gemstone.AssetId] = []

    public init() {}

    public func asset(assetId: Gemstone.AssetId) async throws -> Gemstone.Asset {
        requestedAssetIds.append(assetId)
        if assetId == heldAssetId {
            await Task { [heldAsset] in for await _ in heldAsset {} }.value
        }
        if let asset = assetsById[assetId] {
            return asset
        }
        return try assetResult.get()
    }

    public func releaseHeldAsset() {
        releaseHeld.finish()
    }

    public func enableAsset(walletId _: Gemstone.WalletId, assetId: Gemstone.AssetId) async throws {
        enabledAssetIds.append(assetId)
        if let enableAssetError {
            throw enableAssetError
        }
    }

    public func updateAsset(assetId: Gemstone.AssetId) async throws {
        updatedAssetIds.append(assetId)
        if let updateError {
            throw updateError
        }
    }

    public func assetState(asset: Gemstone.Asset) -> GemReceiveAssetState {
        GemReceiveAssetState(asset: Gemstone.assetText(asset: asset), warnings: warningsValue)
    }
}

public final class GemTransactionsServiceMock: GemTransactionsServiceProtocol, @unchecked Sendable {
    public var filterChainsValue: [Gemstone.Chain] = []
    public var refreshState: GemLoadState = .data
    public var loadState: GemLoadState = .data

    public private(set) var syncedAssetIds: [Gemstone.AssetId?] = []
    public private(set) var loadedTransactionIds: [Gemstone.TransactionId] = []

    public init(filterChains: [Gemstone.Chain] = []) {
        filterChainsValue = filterChains
    }

    public func filterChains(wallet _: Gemstone.Wallet) -> [Gemstone.Chain] {
        filterChainsValue
    }

    public func refresh(assetId: Gemstone.AssetId?) async -> GemLoadState {
        syncedAssetIds.append(assetId)
        return refreshState
    }

    public func loadTransaction(walletId _: Gemstone.WalletId, transactionId: Gemstone.TransactionId) async -> GemLoadState {
        loadedTransactionIds.append(transactionId)
        return loadState
    }
}

private extension GemStakeAmountInput {
    var selectedValidatorId: String? {
        switch self {
        case let .stake(validator), let .redelegate(_, validator), let .rewards(_, validator): validator.id
        case let .unstake(delegation), let .withdraw(delegation): delegation.validator.id
        case .freeze, .unfreeze: nil
        }
    }
}
