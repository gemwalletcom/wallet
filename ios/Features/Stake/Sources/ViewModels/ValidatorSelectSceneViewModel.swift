// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemStakeAmountInput
import protocol Gemstone.GemStakeServiceProtocol
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
    private let currentValidatorId: String
    private let selectValidator: (GemValidatorRow) -> Void

    public init(
        service: any GemStakeServiceProtocol,
        chain: Chain,
        input: GemStakeAmountInput,
        currentValidatorId: String,
        selectValidator: @escaping (GemValidatorRow) -> Void,
    ) {
        self.service = service
        self.chain = chain
        self.input = input
        self.currentValidatorId = currentValidatorId
        self.selectValidator = selectValidator
        validatorsQuery = ObservableQuery(ValidatorsQuery(chain: chain, providerType: .stake), initialValue: [])
    }

    public var title: String {
        Localized.Stake.validators
    }

    public var emptyContent: EmptyStateViewModel {
        EmptyStateViewModel(kind: .validators)
    }

    public var list: [ListItemValueSection<GemValidatorRow>] {
        service.stakeValidatorOptions(chain: chain.rawValue, input: input, validators: validatorsQuery.value.map { $0.toGem() }).sections.map {
            ListItemValueSection(section: $0.kind.title, values: $0.rows.map { ListItemValue(value: $0) })
        }
    }

    public func isSelected(_ row: GemValidatorRow) -> Bool {
        row.validator.id == currentValidatorId
    }

    public func onSelect(_ row: GemValidatorRow) {
        selectValidator(row)
    }

    public func explorerContext(for row: GemValidatorRow) -> ExplorerContextData? {
        let validator = row.validator.toPrimitives()
        return row.explorer.map {
            ExplorerContextData(copyValue: .address(value: validator.id, chain: validator.chain), explorerLink: $0.toPrimitives())
        }
    }
}

extension GemValidatorRow: @retroactive Identifiable {
    public var id: String {
        validator.id
    }
}
