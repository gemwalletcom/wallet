// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemAmountRequest
import enum Gemstone.GemStakeAmountInput
import protocol Gemstone.GemStakeServiceProtocol
import struct Gemstone.GemValidatorRow
import GemstonePrimitives
import Localization
import Primitives
import Stake

public enum AmountStakeSelection {
    case validator(ValidatorViewModel, destination: DelegationValidator?)
    case resource(options: [Resource], selected: Resource)
}

@Observable
public final class AmountStakeViewModel {
    let asset: Asset
    public private(set) var stakeInput: GemStakeAmountInput
    private let service: any GemStakeServiceProtocol

    init(asset: Asset, type: GemStakeAmountInput, service: any GemStakeServiceProtocol) {
        self.asset = asset
        self.service = service
        stakeInput = type
    }

    public var selection: AmountStakeSelection {
        switch service.stakeAmountSelection(chain: asset.chain.rawValue, input: stakeInput) {
        case let .validator(validator, canSelect):
            .validator(ValidatorViewModel(row: validator), destination: canSelect ? validator.validator.toPrimitives() : nil)
        case let .resource(options, selected):
            .resource(options: options.map { $0.toPrimitives() }, selected: selected.toPrimitives())
        }
    }

    var validatorTitle: String {
        Localized.Stake.validator
    }

    var request: GemAmountRequest {
        .stake(input: stakeInput)
    }

    func select(_ row: GemValidatorRow) {
        stakeInput = stakeInput.withValidator(validator: row.validator)
    }

    func select(_ resource: Resource) {
        stakeInput = stakeInput.withResource(resource: resource.toGem())
    }
}
