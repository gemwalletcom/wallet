// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemStakeServiceProtocol
import struct Gemstone.GemValidatorRow
import Components
import GemstonePrimitives
import Foundation
import Localization
import Primitives
import PrimitivesComponents

@Observable
public final class ValidatorSelectSceneViewModel {
    public let currentValidator: DelegationValidator?
    private let recommended: [DelegationValidator]
    private let validators: [DelegationValidator]
    public var selectValidator: ((DelegationValidator) -> Void)?
    private let service: any GemStakeServiceProtocol
    private let rowsById: [String: GemValidatorRow]
    private let explorerLinksById: [String: BlockExplorerLink]

    public init(
        service: any GemStakeServiceProtocol,
        currentValidator: DelegationValidator?,
        recommended: [DelegationValidator],
        validators: [DelegationValidator],
        selectValidator: ((DelegationValidator) -> Void)? = nil,
    ) {
        self.service = service
        self.currentValidator = currentValidator
        self.recommended = recommended
        self.validators = validators
        self.selectValidator = selectValidator

        let all = recommended + validators
        rowsById = Dictionary(
            zip(all.map(\.id), service.validatorRows(validators: all.map { $0.toGem() })),
            uniquingKeysWith: { first, _ in first },
        )
        explorerLinksById = Dictionary(
            all.compactMap { validator in service.validatorUrl(validator: validator.toGem()).map { (validator.id, $0.toPrimitives()) } },
            uniquingKeysWith: { first, _ in first },
        )
    }

    public var title: String {
        Localized.Stake.validators
    }

    public var list: [ListItemValueSection<DelegationValidator>] {
        [
            listSection(title: Localized.Common.recommended, validators: recommended),
            listSection(title: Localized.Stake.active, validators: validators),
        ].filter(\.values.isNotEmpty)
    }

    public func explorerLink(for validator: DelegationValidator) -> BlockExplorerLink? {
        explorerLinksById[validator.id]
    }

    public func explorerContext(for validator: DelegationValidator) -> ExplorerContextData? {
        explorerLink(for: validator).map {
            ExplorerContextData(copyValue: .address(value: validator.id, chain: validator.chain), explorerLink: $0)
        }
    }

    public func listSection(title: String, validators: [DelegationValidator]) -> ListItemValueSection<DelegationValidator> {
        ListItemValueSection(
            section: title,
            values: validators.map(listItem),
        )
    }

    public func listItem(validator: DelegationValidator) -> ListItemValue<DelegationValidator> {
        let model = ValidatorViewModel(row: validatorRow(for: validator))
        return ListItemValue(
            title: model.name,
            subtitle: model.aprModel.text,
            value: validator,
        )
    }

    public func validatorModel(for validator: DelegationValidator) -> ValidatorViewModel {
        ValidatorViewModel(row: validatorRow(for: validator))
    }

    public func validatorRow(for validator: DelegationValidator) -> GemValidatorRow {
        rowsById[validator.id] ?? service.validatorRow(validator: validator.toGem())
    }
}
