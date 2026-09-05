// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import enum Gemstone.GemAmountType
import enum Gemstone.GemAmountStakeType
import protocol Gemstone.GemAmountServiceProtocol
import struct Gemstone.GemPaymentRecipient
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Stake
import Validators
import struct Gemstone.GemRecipient
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
        let stakeType: GemAmountStakeType = switch action {
        case .stake: .stake
        case let .unstake(delegation): .unstake(delegation: delegation.json())
        case let .redelegate(delegation, _, _): .redelegate(delegation: delegation.json())
        case let .withdraw(delegation): .withdraw(delegation: delegation.json())
        case let .claimRewards(delegations): .rewards(delegations: selectedRewardsDelegations(delegations).map { $0.json() })
        case .freeze: .freeze(resource: selectedResource.map())
        case .unfreeze: .unfreeze(resource: selectedResource.map())
        }
        return .stake(stakeType: stakeType)
    }

    private var selectedResource: Resource {
        guard case let .resource(state) = selection else { return .bandwidth }
        return state.selected
    }

    private func selectedRewardsDelegations(_ delegations: [Delegation]) -> [Delegation] {
        guard case let .validator(state) = selection else { return [] }
        return delegations.filter { $0.validator.id == state.selected.id }
    }

    func recipientData() -> GemPaymentRecipient {
        switch selection {
        case let .validator(state):
            return GemPaymentRecipient(recipient: GemRecipient(address: state.selected.id, name: state.selected.name))
        case let .resource(state):
            let title = ResourceViewModel(resource: state.selected).title
            return GemPaymentRecipient(recipient: GemRecipient(address: title, name: title))
        }
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) throws -> GemTransferData {
        try service.stakeTransferData(asset: asset.map(), stakeType: getStakeType().json(), value: value, useMaxAmount: useMaxAmount)
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
