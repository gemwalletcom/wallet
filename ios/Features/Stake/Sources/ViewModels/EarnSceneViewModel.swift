// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import GemstoneServices
import Foundation
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store

@MainActor
@Observable
public final class EarnSceneViewModel {
    private let service: any GemStakeServiceProtocol
    private var viewState: StateViewType<Bool> = .loading

    public let wallet: Wallet
    public let asset: Asset

    public let assetQuery: ObservableQuery<AssetRequest>
    public let positionsQuery: ObservableQuery<DelegationsRequest>
    public let providersQuery: ObservableQuery<ValidatorsRequest>

    public var assetData: AssetData {
        assetQuery.value
    }

    public var positions: [Delegation] {
        positionsQuery.value
    }

    public var providers: [DelegationValidator] {
        selectable(providersQuery.value)
    }

    public init(
        wallet: Wallet,
        asset: Asset,
        service: any GemStakeServiceProtocol,
    ) {
        self.wallet = wallet
        self.asset = asset
        self.service = service
        assetQuery = ObservableQuery(AssetRequest(walletId: wallet.id, assetId: asset.id), initialValue: .with(asset: asset))
        positionsQuery = ObservableQuery(
            DelegationsRequest(walletId: wallet.id, assetId: asset.id, providerType: .earn),
            initialValue: [],
        )
        providersQuery = ObservableQuery(
            ValidatorsRequest(chain: asset.id.chain, providerType: .earn),
            initialValue: [],
        )
    }

    var title: String {
        Localized.Common.earn
    }

    private func selectable(_ validators: [DelegationValidator]) -> [DelegationValidator] {
        service.selectableValidators(validators: validators.map { $0.map() }).map { $0.map() }
    }

    var assetModel: AssetViewModel {
        AssetViewModel(asset: asset)
    }

    private var apr: Double? {
        providers.first.map(\.apr).flatMap { $0 > 0 ? $0 : nil }
            ?? assetData.metadata.earnApr
    }

    var aprModel: AprViewModel {
        AprViewModel(apr: apr ?? .zero)
    }

    var showDeposit: Bool {
        wallet.canSign && providers.isNotEmpty
    }

    var depositDestination: AmountInput? {
        guard let provider = providers.first else { return nil }
        return AmountInput(
            type: .earn(.deposit(provider)),
            asset: asset,
        )
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .earn(symbol: asset.symbol))
    }

    var positionModels: [DelegationViewModel] {
        positions
            .filter { (BigInt($0.base.balance) ?? .zero) > 0 }
            .map { DelegationViewModel(service: service, delegation: $0, asset: asset, currencyCode: service.getCurrency()) }
    }

    var hasPositions: Bool {
        positionModels.isNotEmpty
    }

    var showEmptyState: Bool {
        !hasPositions && !viewState.isLoading
    }

    var positionsSectionTitle: String {
        hasPositions ? Localized.Perpetual.positions : .empty
    }

    var providersState: StateViewType<Bool> {
        switch viewState {
        case .noData: .noData
        case .loading: providers.isEmpty ? .loading : .data(true)
        case .data: providers.isEmpty ? .noData : .data(true)
        case let .error(error): .error(error)
        }
    }
}

// MARK: - Actions

extension EarnSceneViewModel {
    func load() async {
        viewState = .loading
        do {
            try await service.syncEarn(assetId: asset.id.identifier)
            viewState = .data(true)
        } catch {
            viewState = .error(error)
        }
    }
}
