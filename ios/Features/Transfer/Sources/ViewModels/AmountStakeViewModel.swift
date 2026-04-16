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

public final class AmountStakeViewModel: AmountDataProvidable {
    let asset: Asset
    let action: StakeAmountType
    public let validatorSelection: SelectionState<DelegationValidator>

    init(asset: Asset, action: StakeAmountType) {
        self.asset = asset
        self.action = action
        validatorSelection = Self.makeValidatorSelection(action: action)
    }

    private static func makeValidatorSelection(action: StakeAmountType) -> SelectionState<DelegationValidator> {
        switch action {
        case let .stake(validators, recommended):
            SelectionState(options: validators, selected: selectedValidator(from: validators, recommended: recommended), isEnabled: true, title: Localized.Stake.validator)
        case let .unstake(delegation):
            SelectionState(options: [delegation.validator], selected: delegation.validator, isEnabled: false, title: Localized.Stake.validator)
        case let .redelegate(_, validators, recommended):
            SelectionState(options: validators, selected: selectedValidator(from: validators, recommended: recommended), isEnabled: true, title: Localized.Stake.validator)
        case let .withdraw(delegation):
            SelectionState(options: [delegation.validator], selected: delegation.validator, isEnabled: false, title: Localized.Stake.validator)
        case let .claimRewards(delegations):
            SelectionState(options: delegations.map(\.validator), selected: selectedClaimRewardsValidator(from: delegations), isEnabled: delegations.count > 1, title: Localized.Stake.validator)
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
        case .unstake, .withdraw, .claimRewards: .unstake
        }
    }

    var title: String {
        switch action {
        case .stake: Localized.Transfer.Stake.title
        case .unstake: Localized.Transfer.Unstake.title
        case .redelegate: Localized.Transfer.Redelegate.title
        case .withdraw: Localized.Transfer.Withdraw.title
        case .claimRewards: Localized.Transfer.ClaimRewards.title
        }
    }

    var amountType: AmountType {
        .stake(action)
    }

    var minimumValue: BigInt {
        guard let stakeChain = asset.chain.stakeChain else { return .zero }
        return switch action {
        case .stake:
            BigInt(StakeConfig.config(chain: stakeChain).minAmount)
        case .redelegate:
            stakeChain == .smartChain ? BigInt(StakeConfig.config(chain: stakeChain).minAmount) : .zero
        case .unstake:
            .zero
        case .withdraw:
            asset.symbol == "USDC" ? AmountPerpetualLimits.minDeposit : .zero
        case .claimRewards:
            .zero
        }
    }

    var canChangeValue: Bool {
        switch action {
        case .stake, .redelegate:
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
        case .unstake, .redelegate, .withdraw, .claimRewards:
            false
        }
    }

    var reserveForFee: BigInt {
        switch action {
        case .stake where asset.chain != .tron:
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
            return delegations.first { $0.validator.id == validatorSelection.selected.id }?.base.rewardsValue ?? .zero
        }
    }

    func recipientData() -> RecipientData {
        RecipientData(
            recipient: Recipient(
                name: validatorSelection.selected.name,
                address: validatorSelection.selected.id,
                memo: Localized.Stake.viagem,
            ),
            amount: nil,
        )
    }

    func makeTransferData(value: BigInt) throws -> TransferData {
        let stakeType: StakeType = switch action {
        case .stake:
            .stake(validatorSelection.selected)
        case let .unstake(delegation):
            .unstake(delegation)
        case let .redelegate(delegation, _, _):
            .redelegate(RedelegateData(delegation: delegation, toValidator: validatorSelection.selected))
        case let .withdraw(delegation):
            .withdraw(delegation)
        case .claimRewards:
            .rewards([validatorSelection.selected])
        }
        return TransferData(
            type: .stake(asset, stakeType),
            recipientData: recipientData(),
            value: value,
            canChangeValue: canChangeValue,
        )
    }
}
