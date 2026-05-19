// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import Gemstone
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Stake
import Validators

public enum AmountStakeSelection {
    case validator(SelectionState<DelegationValidator>)
    case resource(SelectionState<Resource>)
}

public final class AmountStakeViewModel: AmountDataProvidable {
    let asset: Asset
    let action: AmountStakeType
    public let selection: AmountStakeSelection

    init(asset: Asset, type: AmountStakeType) {
        self.asset = asset
        action = type
        selection = Self.makeSelection(type: type)
    }

    private static func makeSelection(type: AmountStakeType) -> AmountStakeSelection {
        switch type {
        case let .stake(validators, recommended):
            .validator(SelectionState(options: validators, selected: selectedValidator(from: validators, recommended: recommended), isEnabled: true, title: Localized.Stake.validator))
        case let .unstake(delegation):
            .validator(SelectionState(options: [delegation.validator], selected: delegation.validator, isEnabled: false, title: Localized.Stake.validator))
        case let .redelegate(_, validators, recommended):
            .validator(SelectionState(options: validators, selected: selectedValidator(from: validators, recommended: recommended), isEnabled: true, title: Localized.Stake.validator))
        case let .withdraw(delegation):
            .validator(SelectionState(options: [delegation.validator], selected: delegation.validator, isEnabled: false, title: Localized.Stake.validator))
        case let .claimRewards(delegations):
            .validator(SelectionState(options: delegations.map(\.validator), selected: selectedClaimRewardsValidator(from: delegations), isEnabled: delegations.count > 1, title: Localized.Stake.validator))
        case let .freeze(resource), let .unfreeze(resource):
            .resource(SelectionState(options: [.bandwidth, .energy], selected: resource, isEnabled: true, title: Localized.Stake.resource))
        }
    }

    private static func selectedClaimRewardsValidator(from delegations: [Delegation]) -> DelegationValidator {
        guard let first = delegations.first?.validator else {
            preconditionFailure("Claim rewards selection requires at least one delegation")
        }
        return first
    }

    private static func selectedValidator(
        from validators: [DelegationValidator],
        recommended: DelegationValidator?,
    ) -> DelegationValidator {
        if let recommended {
            return recommended
        }

        guard let selected = validators.first else {
            preconditionFailure("Stake validator selection requires at least one validator")
        }

        return selected
    }

    public var validatorSelectType: ValidatorSelectType {
        switch action {
        case .stake, .redelegate: .stake
        case .unstake, .withdraw, .claimRewards, .freeze, .unfreeze: .unstake
        }
    }

    var title: String {
        switch action {
        case .stake: Localized.Transfer.Stake.title
        case .unstake: Localized.Transfer.Unstake.title
        case .redelegate: Localized.Transfer.Redelegate.title
        case .withdraw: Localized.Transfer.Withdraw.title
        case .claimRewards: Localized.Transfer.ClaimRewards.title
        case .freeze: Localized.Transfer.Freeze.title
        case .unfreeze: Localized.Transfer.Unfreeze.title
        }
    }

    var amountType: AmountType {
        switch selection {
        case .validator: .stake(action)
        case let .resource(state):
            switch action {
            case .freeze: .stake(.freeze(state.selected))
            case .unfreeze: .stake(.unfreeze(state.selected))
            default: .stake(action)
            }
        }
    }

    var minimumValue: BigInt {
        guard let stakeChain = asset.chain.stakeChain else { return .zero }
        return switch action {
        case .stake, .freeze:
            BigInt(StakeConfig.config(chain: stakeChain).minAmount)
        case .redelegate:
            stakeChain == .smartChain ? BigInt(StakeConfig.config(chain: stakeChain).minAmount) : .zero
        case .unstake, .unfreeze, .claimRewards:
            .zero
        case .withdraw:
            asset.symbol == "USDC" ? AmountPerpetualLimits.minDeposit : .zero
        }
    }

    var canChangeValue: Bool {
        switch action {
        case .stake, .redelegate, .freeze, .unfreeze:
            true
        case .unstake:
            StakeChain(rawValue: asset.chain.rawValue)?.canChangeAmountOnUnstake ?? true
        case .withdraw, .claimRewards:
            false
        }
    }

    var showsAssetBalance: Bool {
        switch action {
        case .claimRewards: true
        default: canChangeValue
        }
    }

    func shouldReserveFee(from assetData: AssetData) -> Bool {
        let maxAfterFee = max(.zero, availableValue(from: assetData) - reserveForFee)
        return switch action {
        case .stake:
            asset.chain != .tron && maxAfterFee > minimumValue && !reserveForFee.isZero
        case .freeze:
            maxAfterFee > minimumValue
        case .unstake, .redelegate, .withdraw, .claimRewards, .unfreeze:
            false
        }
    }

    var reserveForFee: BigInt {
        switch action {
        case .stake where asset.chain != .tron, .freeze:
            BigInt(Gemstone.Config.shared.getStakeConfig(chain: asset.chain.rawValue).reservedForFees)
        default:
            .zero
        }
    }

    func availableValue(from assetData: AssetData) -> BigInt {
        switch action {
        case .stake:
            if asset.chain == .tron {
                let staked = BigNumberFormatter.standard.number(
                    from: Int(assetData.balance.metadata?.votes ?? 0),
                    decimals: Int(assetData.asset.decimals),
                )
                return (assetData.balance.frozen + assetData.balance.locked) - staked
            }
            return assetData.balance.available
        case let .unstake(delegation), let .redelegate(delegation, _, _), let .withdraw(delegation):
            return delegation.base.balanceValue
        case let .claimRewards(delegations):
            guard case let .validator(state) = selection else { return .zero }
            return delegations.first { $0.validator.id == state.selected.id }?.base.rewardsValue ?? .zero
        case .freeze:
            return assetData.balance.available
        case .unfreeze:
            guard case let .resource(state) = selection else { return .zero }
            return state.selected == .bandwidth ? assetData.balance.frozen : assetData.balance.locked
        }
    }

    func recipientData() -> RecipientData {
        switch selection {
        case let .validator(state):
            return RecipientData(
                recipient: Recipient(
                    name: state.selected.name,
                    address: state.selected.id,
                    memo: nil,
                ),
                amount: nil,
            )
        case let .resource(state):
            let title = ResourceViewModel(resource: state.selected).title
            return RecipientData(
                recipient: Recipient(name: title, address: title, memo: nil),
                amount: nil,
            )
        }
    }

    func makeTransferData(value: BigInt) throws -> TransferData {
        try TransferData(
            type: .stake(asset, getStakeType()),
            recipientData: recipientData(),
            value: value,
            canChangeValue: canChangeValue,
        )
    }

    private func getStakeType() throws -> StakeType {
        switch (action, selection) {
        case let (.stake, .validator(state)):
            .stake(state.selected)
        case let (.unstake(delegation), _):
            .unstake(delegation)
        case let (.redelegate(delegation, _, _), .validator(state)):
            .redelegate(RedelegateData(delegation: delegation, toValidator: state.selected))
        case let (.withdraw(delegation), _):
            .withdraw(delegation)
        case let (.claimRewards, .validator(state)):
            .rewards([state.selected])
        case let (.freeze, .resource(state)):
            .freeze(state.selected)
        case let (.unfreeze, .resource(state)):
            .unfreeze(state.selected)
        default:
            throw AnyError("Unsupported stake selection")
        }
    }
}
