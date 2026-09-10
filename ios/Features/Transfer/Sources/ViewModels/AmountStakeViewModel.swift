// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemAmountType
import protocol Gemstone.GemAmountServiceProtocol
import enum Gemstone.GemStakeAmountInput
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
    let action: GemStakeAmountInput
    public let selection: AmountStakeSelection
    public let recommendedValidators: [DelegationValidator]
    private let service: any GemAmountServiceProtocol

    init(asset: Asset, type: GemStakeAmountInput, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.service = service
        action = type
        switch type {
        case let .freeze(resource), let .unfreeze(resource):
            selection = .resource(SelectionState(options: [.bandwidth, .energy], selected: resource.map(), isEnabled: true, title: Localized.Stake.resource))
            recommendedValidators = []
        case .stake, .unstake, .redelegate, .withdraw, .rewards:
            let validators = service.stakeValidatorSelection(chain: asset.chain.rawValue, input: type)
            let options = validators.options.map { $0.map() }
            guard let selected = validators.validator?.map() ?? options.first else {
                preconditionFailure("Stake action \(type) requires at least one validator")
            }
            selection = .validator(SelectionState(options: options, selected: selected, isEnabled: validators.canSelect, title: Localized.Stake.validator))
            recommendedValidators = validators.recommended.map { $0.map() }
        }
    }

    var title: String {
        switch action {
        case .stake: Localized.Transfer.Stake.title
        case .unstake: Localized.Transfer.Unstake.title
        case .redelegate: Localized.Transfer.Redelegate.title
        case .withdraw: Localized.Transfer.Withdraw.title
        case .rewards: Localized.Transfer.ClaimRewards.title
        case .freeze: Localized.Transfer.Freeze.title
        case .unfreeze: Localized.Transfer.Unfreeze.title
        }
    }

    var amountType: AmountType {
        switch selection {
        case .validator: .stake(action)
        case let .resource(state):
            switch action {
            case .freeze: .stake(.freeze(resource: state.selected.map()))
            case .unfreeze: .stake(.unfreeze(resource: state.selected.map()))
            default: .stake(action)
            }
        }
    }

    var gemAmountType: GemAmountType {
        service.stakeAmountType(stakeType: stakeType.map(), delegations: rewardsDelegations.map { $0.map() })
    }

    private var rewardsDelegations: [Delegation] {
        guard case let .rewards(delegations) = action else { return [] }
        return delegations.map { Delegation(core: $0) }
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) -> GemTransferData {
        service.stakeTransferData(asset: asset.map(), stakeType: stakeType.map(), value: value, useMaxAmount: useMaxAmount)
    }

    private var stakeType: StakeType {
        switch action {
        case .stake: .stake(selectedValidator)
        case let .unstake(delegation): .unstake(Delegation(core: delegation))
        case let .redelegate(_, delegation): .redelegate(RedelegateData(delegation: Delegation(core: delegation), toValidator: selectedValidator))
        case let .withdraw(delegation): .withdraw(Delegation(core: delegation))
        case .rewards: .rewards([selectedValidator])
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
