// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemValidatorRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

@Observable
public final class ValidatorSelectSceneViewModel {
    public let currentValidator: DelegationValidator?
    private let recommended: [GemValidatorRow]
    private let validators: [GemValidatorRow]
    public var selectValidator: ((GemValidatorRow) -> Void)?

    public init(
        currentValidator: DelegationValidator?,
        recommended: [GemValidatorRow],
        validators: [GemValidatorRow],
        selectValidator: ((GemValidatorRow) -> Void)? = nil,
    ) {
        self.currentValidator = currentValidator
        self.recommended = recommended
        self.validators = validators
        self.selectValidator = selectValidator
    }

    public var title: String {
        Localized.Stake.validators
    }

    public var emptyContent: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: EmptyContentType(.validators))
    }

    public var list: [ListItemValueSection<GemValidatorRow>] {
        [
            listSection(title: Localized.Common.recommended, rows: recommended),
            listSection(title: Localized.Stake.active, rows: validators),
        ].filter(\.values.isNotEmpty)
    }

    public func explorerContext(for row: GemValidatorRow) -> ExplorerContextData? {
        let validator = row.validator.toPrimitives()
        return row.explorer.map {
            ExplorerContextData(copyValue: .address(value: validator.id, chain: validator.chain), explorerLink: $0.toPrimitives())
        }
    }

    public func listSection(title: String, rows: [GemValidatorRow]) -> ListItemValueSection<GemValidatorRow> {
        ListItemValueSection(
            section: title,
            values: rows.map(listItem),
        )
    }

    public func listItem(row: GemValidatorRow) -> ListItemValue<GemValidatorRow> {
        let model = ValidatorViewModel(row: row)
        return ListItemValue(
            title: model.name,
            subtitle: model.aprText,
            value: row,
        )
    }

    public func validatorModel(for row: GemValidatorRow) -> ValidatorViewModel {
        ValidatorViewModel(row: row)
    }
}

extension GemValidatorRow: @retroactive Identifiable {
    public var id: String {
        validator.id
    }
}
