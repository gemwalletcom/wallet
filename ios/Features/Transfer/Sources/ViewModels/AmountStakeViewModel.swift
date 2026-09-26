// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemAmountRequest
import enum Gemstone.GemStakeAmountInput
import protocol Gemstone.GemStakeServiceProtocol
import struct Gemstone.GemValidatorRow
import GemstonePrimitives
import Primitives

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
