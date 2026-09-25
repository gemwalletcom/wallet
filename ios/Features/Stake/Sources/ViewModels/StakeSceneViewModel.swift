// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemInfoSheet
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import enum Gemstone.GemLoadState
import struct Gemstone.GemStakeDelegationItem
import enum Gemstone.GemStakeDestination
import struct Gemstone.GemStakeInput
import enum Gemstone.GemStakeSection
import protocol Gemstone.GemStakeServiceProtocol
import struct Gemstone.GemStakeViewState
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstoneServices
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@MainActor
@Observable
public final class StakeSceneViewModel {
    private let service: any GemStakeServiceProtocol
    private let onNavigate: StakeRouteAction

    private var delegationsState: GemLoadState = .loading
    private let chain: StakeChain

    public let wallet: Wallet
    public let delegationsQuery: ObservableQuery<DelegationsQuery>
    public let validatorsQuery: ObservableQuery<ValidatorsQuery>
    public let assetQuery: ObservableQuery<AssetQuery>

    public var assetData: AssetData {
        assetQuery.value
    }

    public var isPresentingInfoSheet: GemInfoSheet? = .none

    public init(
        wallet: Wallet,
        chain: StakeChain,
        service: any GemStakeServiceProtocol,
        onNavigate: StakeRouteAction,
    ) {
        self.wallet = wallet
        self.chain = chain
        self.service = service
        self.onNavigate = onNavigate
        delegationsQuery = ObservableQuery(DelegationsQuery(walletId: wallet.id, assetId: chain.chain.assetId, providerType: .stake), initialValue: [])
        validatorsQuery = ObservableQuery(ValidatorsQuery(chain: chain.chain, providerType: .stake), initialValue: [])
        assetQuery = ObservableQuery(AssetQuery(walletId: wallet.id, assetId: chain.chain.assetId), initialValue: .with(asset: chain.chain.asset))
    }

    var title: String {
        Localized.Transfer.Stake.title
    }

    public var viewState: GemStakeViewState {
        service.stakeViewState(
            input: GemStakeInput(
                walletType: wallet.type.toGem(),
                asset: asset.toGem(),
                balance: GemAssetBalance(assetData.balance, assetId: asset.id, isActive: assetData.metadata.isActive),
                balanceMetadata: assetData.balance.metadata?.toGem(),
                stakingApr: assetData.metadata.stakingApr,
                price: assetData.price?.price,
                currency: service.getCurrency(),
                validators: validatorsQuery.value.map { $0.toGem() },
                delegations: delegationsQuery.value.map { $0.toGem() },
            ),
        )
    }

    func showsDelegationsPlaceholder(_ state: GemStakeViewState) -> Bool {
        !state.sections.contains(.delegations)
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: EmptyContentType(.stake, symbol: asset.symbol))
    }

    func delegationsViewState(_ state: GemStakeViewState) -> StateViewType<[GemStakeDelegationItem]> {
        delegationsState.stateViewType(state.delegations)
    }

    func route(destination: GemStakeDestination) -> StakeRoute {
        switch destination {
        case let .amount(input): route(amount: .stake(input))
        case let .confirm(transfer): .transfer(.confirm(transfer))
        }
    }
}

// MARK: - Business Logic

extension StakeSceneViewModel {
    func load() async {
        delegationsState = .loading
        delegationsState = await service.refresh(chain: chain.chain.rawValue, delegations: viewState.delegations.map(\.delegation))
    }

    func onSelect(destination: GemStakeDestination) {
        onNavigate?(route(destination: destination))
    }

    func onSelect(delegation item: GemStakeDelegationItem) {
        onNavigate?(item.destination.route(delegation: item.delegation.toPrimitives()))
    }

    func onInfo(_ topic: GemInfoTopic) {
        isPresentingInfoSheet = topic.infoSheet
    }

    func onStakeFrozenInfo() {
        isPresentingInfoSheet = GemInfoTopic.stakeFrozenRequired.infoSheet
    }
}

// MARK: - Private

extension StakeSceneViewModel {
    var assetModel: AssetViewModel {
        AssetViewModel(asset: asset)
    }

    private var asset: Asset {
        chain.chain.asset
    }

    private func route(amount: AmountType) -> StakeRoute {
        .transfer(.amount(AmountInput(type: amount, asset: asset)))
    }
}
