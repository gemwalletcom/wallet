// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemStakeServiceProtocol
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
        service.validatorUrl(validator: validator.map()).map { $0.map() }
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
        let model = ValidatorViewModel(validator: validator)
        return ListItemValue(
            title: model.name,
            subtitle: model.aprModel.text,
            value: validator,
        )
    }
}
