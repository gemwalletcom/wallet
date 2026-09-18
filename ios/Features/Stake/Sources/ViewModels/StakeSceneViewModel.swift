// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemStakeAction
import struct Gemstone.GemStakeActionItem
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemLoadState
import enum Gemstone.GemListRow
import enum Gemstone.GemStakeSection
import struct Gemstone.GemClaimRewards
import struct Gemstone.GemAssetBalance
import protocol Gemstone.GemStakeServiceProtocol
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import GemstoneServices
import Store
import SwiftUI
import struct Gemstone.GemTransferData

@MainActor
@Observable
public final class StakeSceneViewModel {
    private let service: any GemStakeServiceProtocol
    private let onNavigate: StakeRouteAction

    private var delegationsState: GemLoadState = .loading
    private let chain: StakeChain

    private let formatter = ValueFormatter(style: .auto)

    public let wallet: Wallet
    public let delegationsQuery: ObservableQuery<DelegationsRequest>
    public let validatorsQuery: ObservableQuery<ValidatorsRequest>
    public let assetQuery: ObservableQuery<AssetRequest>

    public var delegations: [Delegation] {
        service.sortedDelegations(delegations: delegationsQuery.value.map { $0.toGem() }).map { Delegation(core: $0) }
    }

    public var validators: [DelegationValidator] {
        selectable(validatorsQuery.value)
    }

    public var assetData: AssetData {
        assetQuery.value
    }

    public var isPresentingInfoSheet: InfoSheetType? = .none

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
        delegationsQuery = ObservableQuery(DelegationsRequest(walletId: wallet.id, assetId: chain.chain.assetId, providerType: .stake), initialValue: [])
        validatorsQuery = ObservableQuery(ValidatorsRequest(chain: chain.chain, providerType: .stake), initialValue: [])
        assetQuery = ObservableQuery(AssetRequest(walletId: wallet.id, assetId: chain.chain.assetId), initialValue: .with(asset: chain.chain.asset))
    }

    public var stakeInfoUrl: URL {
        AppUrl.docs(.staking(chain.rawValue))
    }

    var title: String {
        Localized.Transfer.Stake.title
    }

    private func selectable(_ validators: [DelegationValidator]) -> [DelegationValidator] {
        service.selectableValidators(validators: validators.map { $0.toGem() }).map { $0.toPrimitives() }
    }

    var sections: [GemStakeSection] {
        service.stakeSections(chain: chain.chain.rawValue, hasActions: actions.isNotEmpty, hasDelegations: delegations.isNotEmpty)
    }

    var infoRows: [GemListRow] {
        service.stakeInfoRows(asset: asset.toGem(), stakingApr: assetData.metadata.stakingApr)
    }

    var actions: [GemStakeActionItem] {
        stakeActions
    }

    var sectionModels: [StakeSectionViewModel] {
        sections.map { StakeSectionViewModel(section: $0, title: $0.title) }
    }

    var showsDelegationsPlaceholder: Bool {
        !sections.contains(.delegations)
    }

    var actionModels: [StakeActionViewModel] {
        actions.map { item in
            StakeActionViewModel(
                id: String(describing: item.action),
                model: actionListItem(item),
                action: item.action,
                infoAction: frozenBalanceInfoAction(for: item),
                isEnabled: item.isEnabled,
            )
        }
    }

    var energyField: ListItemField {
        ListItemField(title: Resource.energy.title, value: balanceModel.energyText)
    }

    var bandwidthField: ListItemField {
        ListItemField(title: Resource.bandwidth.title, value: balanceModel.bandwidthText)
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .stake(symbol: assetModel.symbol))
    }

    func route(delegation: DelegationViewModel) -> StakeRoute {
        service.delegationDestination(walletType: wallet.type.toGem(), asset: asset.toGem(), delegation: delegation.delegation.toGem())
            .route(delegation: delegation.delegation, validators: validators)
    }

    var delegationsViewState: StateViewType<[DelegationViewModel]> {
        let currency = service.getCurrency().toPrimitives()
        let delegationModels = delegations.map { delegation in
            DelegationViewModel(
                service: service,
                delegation: delegation,
                asset: asset,
                currency: currency,
            )
        }

        switch delegationsState {
        case .noData: return .noData
        case .loading: return delegationModels.isEmpty ? .loading : .data(delegationModels)
        case .data: return delegationModels.isEmpty ? .noData : .data(delegationModels)
        case let .error(error): return .error(error)
        }
    }

    var claimRewardsRoute: StakeRoute {
        switch claimRewards.destination {
        case let .transfer(transfer): .transfer(.confirm(transfer))
        case let .amount(delegations): route(amount: .stake(.rewards(delegations: delegations, validator: nil)))
        }
    }

    func route(action: GemStakeAction) -> StakeRoute {
        switch action {
        case .stake: route(amount: .stake(.stake(validators: validators.map { $0.toGem() }, validator: nil)))
        case .freeze: route(amount: .stake(.freeze(resource: Resource.bandwidth.toGem())))
        case .unfreeze: route(amount: .stake(.unfreeze(resource: Resource.bandwidth.toGem())))
        case .claimRewards: claimRewardsRoute
        }
    }

    func actionListItem(_ item: GemStakeActionItem) -> ListItemModel {
        if let infoAction = frozenBalanceInfoAction(for: item) {
            return ListItemModel(title: item.action.title, titleStyle: .bodySecondary, infoAction: infoAction)
        }
        return ListItemModel(title: item.action.title, subtitle: item.value?.text())
    }

    func frozenBalanceInfoAction(for item: GemStakeActionItem) -> InfoSheetAction? {
        item.requiresFrozenBalance ? onStakeFrozenInfo : .none
    }
}

// MARK: - Business Logic

extension StakeSceneViewModel {
    func load() async {
        delegationsState = .loading
        delegationsState = await service.refresh(chain: chain.chain.rawValue, delegations: delegations.map { $0.toGem() })
    }

    func onSelect(action: GemStakeAction) {
        onNavigate?(route(action: action))
    }

    func onSelect(delegation: DelegationViewModel) {
        onNavigate?(route(delegation: delegation))
    }

    func onInfo(_ topic: GemInfoTopic) {
        isPresentingInfoSheet = InfoSheetType(topic: topic, assetImage: assetModel.assetImage)
    }

    func onStakeFrozenInfo() {
        isPresentingInfoSheet = .stakeFrozenRequired
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

    private var stakeActions: [GemStakeActionItem] {
        service.stakeActions(
            walletType: wallet.type.toGem(),
            chain: chain.chain.rawValue,
            hasValidators: validators.isNotEmpty,
            balance: GemAssetBalance(assetData.balance, assetId: asset.id, isActive: assetData.metadata.isActive),
            delegations: delegations.map { $0.toGem() },
        )
    }

    private var claimRewards: GemClaimRewards {
        service.claimRewards(chain: chain.chain.rawValue, delegations: delegations.map { $0.toGem() })
    }

    private var balanceModel: BalanceViewModel {
        BalanceViewModel(asset: asset, balance: assetData.balance, formatter: formatter)
    }

    private func route(amount: AmountType) -> StakeRoute {
        .transfer(.amount(AmountInput(type: amount, asset: asset)))
    }
}
