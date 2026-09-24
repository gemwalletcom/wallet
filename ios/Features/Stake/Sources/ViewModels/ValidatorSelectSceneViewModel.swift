// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemStakeServiceProtocol
import struct Gemstone.GemValidatorRow
import func Gemstone.validatorRow
import GemstonePrimitives
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
    }

    public var title: String {
        Localized.Stake.validators
    }

    public var emptyContent: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: EmptyContentType(.validators))
    }

    public var list: [ListItemValueSection<DelegationValidator>] {
        [
            listSection(title: Localized.Common.recommended, validators: recommended),
            listSection(title: Localized.Stake.active, validators: validators),
        ].filter(\.values.isNotEmpty)
    }

    public func explorerContext(for validator: DelegationValidator) -> ExplorerContextData? {
        rowsById[validator.id]?.explorer.map {
            ExplorerContextData(copyValue: .address(value: validator.id, chain: validator.chain), explorerLink: $0.toPrimitives())
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
            subtitle: model.aprText,
            value: validator,
        )
    }

    public func validatorModel(for validator: DelegationValidator) -> ValidatorViewModel {
        ValidatorViewModel(row: validatorRow(for: validator))
    }

    public func validatorRow(for validator: DelegationValidator) -> GemValidatorRow {
        rowsById[validator.id] ?? Gemstone.validatorRow(validator: validator.toGem())
    }
}
