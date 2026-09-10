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
    public let selection: AmountStakeSelection
    public let recommendedValidators: [DelegationValidator]
    private let service: any GemAmountServiceProtocol
    private var action: GemStakeAmountInput

    init(asset: Asset, type: GemStakeAmountInput, service: any GemAmountServiceProtocol) {
        self.asset = asset
        self.service = service
        switch type {
        case let .freeze(resource), let .unfreeze(resource):
            selection = .resource(SelectionState(options: [.bandwidth, .energy], selected: resource.map(), isEnabled: true, title: Localized.Stake.resource))
            recommendedValidators = []
            action = type
        case .stake, .unstake, .redelegate, .withdraw, .rewards:
            let validators = service.stakeValidatorSelection(chain: asset.chain.rawValue, input: type)
            guard let selected = validators.validator else {
                preconditionFailure("Stake action \(type) requires at least one validator")
            }
            selection = .validator(SelectionState(options: validators.options.map { $0.map() }, selected: selected.map(), isEnabled: validators.canSelect, title: Localized.Stake.validator))
            recommendedValidators = validators.recommended.map { $0.map() }
            action = type.withValidator(validator: selected)
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

    var gemAmountType: GemAmountType {
        action.amountType()
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) throws -> GemTransferData {
        service.stakeTransferData(asset: asset.map(), stakeType: try action.stakeType(), value: value, useMaxAmount: useMaxAmount)
    }

    func select(_ validator: DelegationValidator) {
        guard case let .validator(state) = selection else { return }
        state.selected = validator
        action = action.withValidator(validator: validator.map())
    }

    func select(_ resource: Resource) {
        guard case let .resource(state) = selection else { return }
        state.selected = resource
        action = action.withResource(resource: resource.map())
    }
}
