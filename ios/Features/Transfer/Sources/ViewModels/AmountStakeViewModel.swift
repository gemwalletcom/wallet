// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemAmountType
import enum Gemstone.GemStakeAmountInput
import protocol Gemstone.GemStakeServiceProtocol
import struct Gemstone.GemTransferData
import struct Gemstone.GemValidatorRow
import func Gemstone.validatorRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Stake

public enum AmountStakeSelection {
    case validator(SelectionState<GemValidatorRow>)
    case resource(SelectionState<Resource>)
}

public extension SelectionState where T == GemValidatorRow {
    var selectedValidator: DelegationValidator {
        selected.validator.toPrimitives()
    }
}

public final class AmountStakeViewModel: AmountDataProvidable {
    let asset: Asset
    public let selection: AmountStakeSelection
    public let recommendedValidators: [DelegationValidator]
    private let service: any GemStakeServiceProtocol
    private var action: GemStakeAmountInput

    init(asset: Asset, type: GemStakeAmountInput, service: any GemStakeServiceProtocol) {
        self.asset = asset
        self.service = service
        if let resource = type.resource() {
            selection = .resource(
                SelectionState(
                    options: service.resourceOptions(chain: asset.chain.rawValue).map { $0.toPrimitives() },
                    selected: resource.toPrimitives(),
                    isEnabled: true,
                    title: Localized.Stake.resource,
                ),
            )
            recommendedValidators = []
            action = type
        } else {
            let validators = service.stakeValidatorSelection(chain: asset.chain.rawValue, input: type)
            guard let selected = validators.validator else {
                preconditionFailure("Stake action \(type) requires at least one validator")
            }
            selection = .validator(SelectionState(options: validators.options, selected: selected, isEnabled: validators.canSelect, title: Localized.Stake.validator))
            recommendedValidators = validators.recommended.map { $0.validator.toPrimitives() }
            action = type.withValidator(validator: selected.validator)
        }
    }

    var title: String {
        gemAmountType.title().title
    }

    var gemAmountType: GemAmountType {
        action.amountType()
    }

    func makeTransferData(value: BigInt, useMaxAmount: Bool) throws -> GemTransferData {
        try service.stakeTransferData(asset: asset.toGem(), stakeType: action.stakeType(), value: value, useMaxAmount: useMaxAmount)
    }

    func select(_ validator: DelegationValidator) {
        guard case let .validator(state) = selection else { return }
        state.selected = validatorRow(validator: validator.toGem())
        action = action.withValidator(validator: validator.toGem())
    }

    func select(_ resource: Resource) {
        guard case let .resource(state) = selection else { return }
        state.selected = resource
        action = action.withResource(resource: resource.toGem())
    }
}
