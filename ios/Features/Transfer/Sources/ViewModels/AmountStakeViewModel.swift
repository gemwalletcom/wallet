// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemAmountRequest
import enum Gemstone.GemStakeAmountInput
import protocol Gemstone.GemStakeServiceProtocol
import struct Gemstone.GemValidatorRow
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

public final class AmountStakeViewModel {
    let asset: Asset
    public let selection: AmountStakeSelection
    public let recommendedValidators: [GemValidatorRow]
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
            recommendedValidators = validators.recommended
            action = type.withValidator(validator: selected.validator)
        }
    }

    var request: GemAmountRequest {
        .stake(input: action)
    }

    func select(_ row: GemValidatorRow) {
        guard case let .validator(state) = selection else { return }
        state.selected = row
        action = action.withValidator(validator: row.validator)
    }

    func select(_ resource: Resource) {
        guard case let .resource(state) = selection else { return }
        state.selected = resource
        action = action.withResource(resource: resource.toGem())
    }
}
