// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import protocol Gemstone.GemExplorerServiceProtocol
import protocol Gemstone.GemStakeServiceProtocol
import func Gemstone.stakeCanClaimRewards
import func Gemstone.stakeRequiresFrozenBalance
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import GemstoneServices
import Store
import SwiftUI

@MainActor
@Observable
public final class StakeSceneViewModel {
    private let stakeService: any GemStakeServiceProtocol
    private let explorerService: any GemExplorerServiceProtocol

    private var delegationsState: StateViewType<Bool> = .loading
    private let chain: StakeChain

    private let formatter = ValueFormatter(style: .auto)
    private let recommendedValidators = StakeRecommendedValidators()
    private let currencyCode: String

    public let wallet: Wallet
    public let delegationsQuery: ObservableQuery<DelegationsRequest>
    public let validatorsQuery: ObservableQuery<ValidatorsRequest>
    public let assetQuery: ObservableQuery<AssetRequest>

    public var delegations: [Delegation] {
        delegationsQuery.value
    }

    public var validators: [DelegationValidator] {
        validatorsQuery.value
    }

    public var assetData: AssetData {
        assetQuery.value
    }

    public var isPresentingInfoSheet: InfoSheetType? = .none

    public init(
        wallet: Wallet,
        chain: StakeChain,
        currencyCode: String,
        stakeService: any GemStakeServiceProtocol,
        explorerService: any GemExplorerServiceProtocol,
    ) {
        self.wallet = wallet
        self.chain = chain
        self.currencyCode = currencyCode
        self.stakeService = stakeService
        self.explorerService = explorerService
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
        wallet.canSign
    }

    var recommendedCurrentValidator: DelegationValidator? {
        recommendedValidators.randomValidator(chain: chain.chain, from: validators)
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .stake(symbol: assetModel.symbol))
    }

    func navigationDestination(for delegation: DelegationViewModel) -> any Hashable {
        switch delegation.state {
        case .awaitingWithdrawal:
            TransferData(
                type: .stake(asset, .withdraw(delegation.delegation)),
                recipient: Recipient(name: delegation.validatorText, address: delegation.delegation.validator.id, memo: ""),
                value: delegation.delegation.base.balanceValue,
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
        let delegationModels = delegations.map { DelegationViewModel(explorerService: explorerService, delegation: $0, asset: asset, currencyCode: currencyCode) }

        switch delegationsState {
        case .noData: return .noData
        case .loading: return delegationModels.isEmpty ? .loading : .data(delegationModels)
        case .data: return delegationModels.isEmpty ? .noData : .data(delegationModels)
        case let .error(error): return .error(error)
        }
    }

    var claimRewardsText: String {
        formatter.string(rewardsValue, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    var showRewards: Bool {
        stakeCanClaimRewards(chain: chain.chain.rawValue, rewardsAmount: rewardsValue.description)
    }

    var canClaimAllRewards: Bool {
        guard showRewards else { return false }
        return chain.supportClaimAllRewards || delegationsWithRewards.count == 1
    }

    var claimRewardsDestination: any Hashable {
        if canClaimAllRewards {
            let validators = delegationsWithRewards.map(\.validator)
            let recipient = if validators.count == 1, let validator = validators.first {
                Recipient(name: validator.name, address: validator.id, memo: .none)
            } else {
                Recipient(name: .none, address: "", memo: .none)
            }
            return TransferData(
                type: .stake(chain.chain.asset, .rewards(validators)),
                recipient: recipient,
                value: rewardsValue,
            )
        }
        return AmountInput(
            type: .stake(.claimRewards(delegations: delegationsWithRewards)),
            asset: asset,
        )
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
        StakeChain(rawValue: chain.rawValue)?.usesFreeze ?? false
    }

    var showUnfreeze: Bool {
        balanceModel.hasStakingResources
    }

    var isStakeEnabled: Bool {
        validators.isNotEmpty && !stakeFrozenRequired
    }

    var stakeInfoAction: InfoSheetAction? {
        guard stakeFrozenRequired else { return nil }
        return onStakeFrozenInfo
    }

    var showTronResources: Bool {
        balanceModel.hasStakingResources
    }
}

// MARK: - Business Logic

extension StakeSceneViewModel {
    func fetch() async {
        delegationsState = .loading
        do {
            let account = try wallet.account(for: chain.chain)
            try await stakeService.sync(walletId: wallet.id.id, chain: chain.chain.rawValue, address: account.address)
            delegationsState = .data(true)
        } catch {
            debugLog("Stake scene fetch error: \(error)")
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

    private var stakeFrozenRequired: Bool {
        stakeRequiresFrozenBalance(chain: chain.chain.rawValue, frozenAmount: balanceModel.frozenResources.description)
    }

    private var balanceModel: BalanceViewModel {
        BalanceViewModel(asset: asset, balance: assetData.balance, formatter: formatter)
    }

    private var rewardsValue: BigInt {
        delegations.map(\.base.rewardsValue).reduce(0, +)
    }

    private var delegationsWithRewards: [Delegation] {
        delegations.filter { $0.base.rewardsValue > 0 }
    }

    private func destination(type: AmountType) -> any Hashable {
        AmountInput(
            type: type,
            asset: asset,
        )
    }
}
