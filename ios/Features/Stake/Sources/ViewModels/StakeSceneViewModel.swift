// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemDelegationDestination
import enum Gemstone.GemStakeAction
import struct Gemstone.GemStakeActionItem
import enum Gemstone.GemStakeInfoRow
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
        service.selectableValidators(validators: validators.map { $0.toGem() }).map { $0.toPrimitives() }
    }

    var sections: [GemStakeSection] {
        service.stakeSections(chain: chain.chain.rawValue, hasActions: actions.isNotEmpty, hasDelegations: delegations.isNotEmpty)
    }

    var infoRows: [GemStakeInfoRow] {
        service.stakeInfoRows(chain: chain.chain.rawValue, stakingApr: assetData.metadata.stakingApr)
    }

    var actions: [GemStakeActionItem] {
        stakeActions
    }

    var stakeAprModel: AprViewModel {
        AprViewModel(apr: assetData.metadata.stakingApr ?? .zero)
    }

    var energyField: ListItemField {
        ListItemField(title: Resource.energy.title, value: balanceModel.energyText)
    }

    var bandwidthField: ListItemField {
        ListItemField(title: Resource.bandwidth.title, value: balanceModel.bandwidthText)
    }

    func infoField(for row: GemStakeInfoRow) -> ListItemField {
        switch row {
        case .apr: ListItemField(title: stakeAprModel.title, value: stakeAprModel.subtitle)
        case .lockTime: ListItemField(title: row.title, value: lockTimeValue)
        case .minimumAmount:
            ListItemField(title: row.title, value: formatter.string(service.minStakeAmount(chain: chain.chain.rawValue), decimals: Int(asset.decimals), currency: asset.symbol))
        }
    }

    func infoAction(for row: GemStakeInfoRow) -> InfoSheetAction? {
        switch row {
        case .apr: onAprInfo
        case .lockTime: onLockTimeInfo
        case .minimumAmount: nil
        }
    }

    private var lockTimeValue: String {
        let now = Date.now
        let date = now.addingTimeInterval(TimeInterval(service.lockTimeSeconds(chain: chain.chain.rawValue)))
        return Self.lockTimeFormatter.string(from: now, to: date) ?? .empty
    }

    var lockTimeInfoSheet: InfoSheetType {
        InfoSheetType.stakeLockTime(assetModel.assetImage.placeholder)
    }

    var aprInfoSheet: InfoSheetType {
        InfoSheetType.stakeApr(assetModel.assetImage.placeholder)
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .stake(symbol: assetModel.symbol))
    }

    func navigationDestination(for delegation: DelegationViewModel) -> any Hashable {
        switch delegation.destination {
        case let .withdraw(transfer): transfer
        case .details: delegation.delegation
        }
    }

    private func destination(for delegation: Delegation) -> GemDelegationDestination {
        service.delegationDestination(walletType: wallet.type.toGem(), asset: asset.toGem(), delegation: delegation.toGem())
    }

    var delegationsViewState: StateViewType<[DelegationViewModel]> {
        let currency = service.getCurrency().toPrimitives()
        let delegationModels = delegations.map { delegation in
            DelegationViewModel(
                service: service,
                delegation: delegation,
                asset: asset,
                currency: currency,
                destination: destination(for: delegation),
            )
        }

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

    var claimRewardsDestination: any Hashable {
        switch claimRewards.destination {
        case let .transfer(transfer): transfer
        case let .amount(delegations): AmountInput(type: .stake(.rewards(delegations: delegations, validator: nil)), asset: asset)
        }
    }

    func destination(for action: GemStakeAction) -> any Hashable {
        switch action {
        case .stake: destination(type: .stake(.stake(validators: validators.map { $0.toGem() }, validator: nil)))
        case .freeze: destination(type: .stake(.freeze(resource: Resource.bandwidth.toGem())))
        case .unfreeze: destination(type: .stake(.unfreeze(resource: Resource.bandwidth.toGem())))
        case .claimRewards: claimRewardsDestination
        }
    }

    func subtitle(for action: GemStakeAction) -> String? {
        action == .claimRewards ? claimRewardsText : .none
    }

    func frozenBalanceInfoAction(for item: GemStakeActionItem) -> InfoSheetAction? {
        item.requiresFrozenBalance ? onStakeFrozenInfo : .none
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

    private func destination(type: AmountType) -> any Hashable {
        AmountInput(
            type: type,
            asset: asset,
        )
    }
}
