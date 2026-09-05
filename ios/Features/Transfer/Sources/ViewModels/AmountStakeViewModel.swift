// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import enum Gemstone.GemAmountType
import protocol Gemstone.GemAmountServiceProtocol
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Stake
import Validators
import struct Gemstone.GemTransferData

public enum AmountStakeSelection {
    case validator(SelectionState<DelegationValidator>)
    case resource(SelectionState<Resource>)
}

public final class AmountStakeViewModel: AmountDataProvidable {
    let asset: Asset
    let action: AmountStakeType
    public let selection: AmountStakeSelection
    private let service: any GemAmountServiceProtocol

    init(asset: Asset, type: AmountStakeType, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.service = service
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

    var gemAmountType: GemAmountType {
        service.stakeAmountType(stakeType: stakeType.json(), delegations: rewardsDelegations.map { $0.json() })
    }

    private var rewardsDelegations: [Delegation] {
        guard case let .claimRewards(delegations) = action else { return [] }
        return delegations
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) -> GemTransferData {
        service.stakeTransferData(asset: asset.map(), stakeType: stakeType.json(), value: value, useMaxAmount: useMaxAmount)
    }

    private var stakeType: StakeType {
        switch action {
        case .stake: .stake(selectedValidator)
        case let .unstake(delegation): .unstake(delegation)
        case let .redelegate(delegation, _, _): .redelegate(RedelegateData(delegation: delegation, toValidator: selectedValidator))
        case let .withdraw(delegation): .withdraw(delegation)
        case .claimRewards: .rewards([selectedValidator])
        case .freeze: .freeze(selectedResource)
        case .unfreeze: .unfreeze(selectedResource)
        }
    }

    private var selectedValidator: DelegationValidator {
        guard case let .validator(state) = selection else {
            preconditionFailure("Stake action \(action) requires a validator selection")
        }
        return state.selected
    }

    private var selectedResource: Resource {
        guard case let .resource(state) = selection else { return .bandwidth }
        return state.selected
    }
}
