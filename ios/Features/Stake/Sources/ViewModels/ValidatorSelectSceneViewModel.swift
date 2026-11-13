// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemStakeAmountInput
import protocol Gemstone.GemStakeServiceProtocol
import struct Gemstone.GemGroupList
import struct Gemstone.GemValidatorRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Store

@MainActor
@Observable
public final class ValidatorSelectSceneViewModel {
    public let validatorsQuery: ObservableQuery<ValidatorsQuery>

    private let service: any GemStakeServiceProtocol
    private let chain: Chain
    private let input: GemStakeAmountInput
    private let selectValidator: (GemValidatorRow) -> Void

    public init(
        service: any GemStakeServiceProtocol,
        chain: Chain,
        input: GemStakeAmountInput,
        selectValidator: @escaping (GemValidatorRow) -> Void,
    ) {
        self.service = service
        self.chain = chain
        self.input = input
        self.selectValidator = selectValidator
        validatorsQuery = ObservableQuery(ValidatorsQuery(chain: chain, providerType: .stake), initialValue: [])
    }

    var title: String {
        Localized.Stake.validators
    }

    var options: GemGroupList {
        service.stakeValidatorOptions(
            chain: chain.rawValue,
            input: input,
            validators: validatorsQuery.value.map { $0.toGem() },
        )
    }

    func onSelect(_ row: GemValidatorRow) {
        selectValidator(row)
    }
}

extension GemValidatorRow: @retroactive Identifiable {
    public var id: String {
        validator.id
    }
}
