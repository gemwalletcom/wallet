// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemStakeAction
import struct Gemstone.GemStakeActionItem
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

    private var delegationsState: StateViewType<Bool> = .loading
    private let chain: StakeChain

    private let formatter = ValueFormatter(style: .auto)

    public let wallet: Wallet
    public let delegationsQuery: ObservableQuery<DelegationsRequest>
    public let validatorsQuery: ObservableQuery<ValidatorsRequest>
    public let assetQuery: ObservableQuery<AssetRequest>

    public var delegations: [Delegation] {
        delegationsQuery.value
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
    ) {
        self.wallet = wallet
        self.chain = chain
        self.service = service
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
        service.selectableValidators(validators: validators.map { $0.map() }).map { $0.map() }
    }

    var stakeTitle: String {
        Localized.Transfer.Stake.title
    }

    var rewardsTitle: String {
        Localized.Transfer.ClaimRewards.title
    }

    var delegationsTitle: String {
        Localized.Stake.delegations
    }

    var stakeAprModel: AprViewModel {
        AprViewModel(apr: assetData.metadata.stakingApr ?? .zero)
    }

    var resourcesTitle: String {
        Localized.Asset.resources
    }

    var energyField: ListItemField {
        ListItemField(title: ResourceViewModel(resource: .energy).title, value: balanceModel.energyText)
    }

    var bandwidthField: ListItemField {
        ListItemField(title: ResourceViewModel(resource: .bandwidth).title, value: balanceModel.bandwidthText)
    }

    var freezeTitle: String {
        Localized.Transfer.Freeze.title
    }

    var unfreezeTitle: String {
        Localized.Transfer.Unfreeze.title
    }

    var lockTimeField: ListItemField {
        let now = Date.now
        let date = now.addingTimeInterval(chain.lockTime)
        let value = Self.lockTimeFormatter.string(from: now, to: date) ?? .empty
        return ListItemField(title: Localized.Stake.lockTime, value: value)
    }

    var lockTimeInfoSheet: InfoSheetType {
        InfoSheetType.stakeLockTime(assetModel.assetImage.placeholder)
    }

    var aprInfoSheet: InfoSheetType {
        InfoSheetType.stakeApr(assetModel.assetImage.placeholder)
    }

    var minAmountField: ListItemField? {
        guard chain.minAmount != 0 else { return .none }
        let value = formatter.string(chain.minAmount, decimals: Int(asset.decimals), currency: asset.symbol)
        return ListItemField(title: Localized.Stake.minimumAmount, value: value)
    }

    var showManage: Bool {
        stakeActions.isNotEmpty
    }

    var recommendedCurrentValidator: DelegationValidator? {
        service.recommendedValidator(chain: chain.chain.rawValue, validators: validators.map { $0.map() }).map { $0.map() }
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .stake(symbol: assetModel.symbol))
    }

    func navigationDestination(for delegation: DelegationViewModel) -> any Hashable {
        switch delegation.state {
        case .awaitingWithdrawal:
            service.stakeTransferData(
                asset: asset.map(),
                stakeType: StakeType.withdraw(delegation.delegation).json(),
                value: delegation.delegation.base.balanceValue,
                useMaxAmount: false,
            )
        case .active, .pending, .inactive, .activating, .deactivating:
            delegation.delegation
        }
    }

    var delegationsSectionTitle: String {
        guard case let .data(delegations) = delegationsViewState, delegations.isNotEmpty else {
            return .empty
        }
        return delegationsTitle
    }

    var delegationsViewState: StateViewType<[DelegationViewModel]> {
        let delegationModels = delegations.map { DelegationViewModel(service: service, delegation: $0, asset: asset, currencyCode: service.getCurrency()) }

        switch delegationsState {
        case .noData: return .noData
        case .loading: return delegationModels.isEmpty ? .loading : .data(delegationModels)
        case .data: return delegationModels.isEmpty ? .noData : .data(delegationModels)
        case let .error(error): return .error(error)
        }
    }

    var claimRewardsText: String {
        formatter.string(claimRewards.value, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    var showRewards: Bool {
        stakeAction(.claimRewards) != nil
    }

    var claimRewardsDestination: any Hashable {
        switch claimRewards.destination {
        case let .transfer(transfer): transfer
        case let .amount(delegations): AmountInput(type: .stake(.claimRewards(delegations: delegations.map { Delegation(core: $0) })), asset: asset)
        }
    }

    var stakeDestination: any Hashable {
        destination(
            type: .stake(.stake(
                validators: validators,
                recommended: recommendedCurrentValidator,
            )),
        )
    }

    var freezeDestination: any Hashable {
        destination(type: .stake(.freeze(.bandwidth)))
    }

    var unfreezeDestination: any Hashable {
        destination(type: .stake(.unfreeze(.bandwidth)))
    }

    var showFreeze: Bool {
        stakeAction(.freeze) != nil
    }

    var showUnfreeze: Bool {
        stakeAction(.unfreeze) != nil
    }

    var isStakeEnabled: Bool {
        stakeAction(.stake)?.isEnabled ?? false
    }

    var stakeInfoAction: InfoSheetAction? {
        guard stakeAction(.stake)?.requiresFrozenBalance == true else { return nil }
        return onStakeFrozenInfo
    }

    var showTronResources: Bool {
        balanceModel.hasStakingResources
    }
}

// MARK: - Business Logic

extension StakeSceneViewModel {
    func load() async {
        delegationsState = .loading
        do {
            try await service.sync(chain: chain.chain.rawValue)
            delegationsState = .data(true)
        } catch {
            debugLog("Stake scene load error: \(error)")
            delegationsState = .error(error)
        }
    }

    func onLockTimeInfo() {
        isPresentingInfoSheet = lockTimeInfoSheet
    }

    func onAprInfo() {
        isPresentingInfoSheet = aprInfoSheet
    }

    func onStakeFrozenInfo() {
        isPresentingInfoSheet = .stakeFrozenRequired
    }
}

// MARK: - Private

extension StakeSceneViewModel {
    private static let lockTimeFormatter: DateComponentsFormatter = {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day]
        formatter.unitsStyle = .full
        return formatter
    }()

    var assetModel: AssetViewModel {
        AssetViewModel(asset: asset)
    }

    private var asset: Asset {
        chain.chain.asset
    }

    private var stakeActions: [GemStakeActionItem] {
        service.stakeActions(
            walletType: wallet.type.map(),
            chain: chain.chain.rawValue,
            hasValidators: validators.isNotEmpty,
            balance: GemAssetBalance(assetData.balance, assetId: asset.id),
            delegations: delegations.map { $0.json() },
        )
    }

    private var claimRewards: GemClaimRewards {
        service.claimRewards(chain: chain.chain.rawValue, delegations: delegations.map { $0.json() })
    }

    private func stakeAction(_ action: GemStakeAction) -> GemStakeActionItem? {
        stakeActions.first { $0.action == action }
    }

    private var balanceModel: BalanceViewModel {
        BalanceViewModel(asset: asset, balance: assetData.balance, formatter: formatter)
    }

    private func destination(type: AmountType) -> any Hashable {
        AmountInput(
            type: type,
            asset: asset,
        )
    }
}
