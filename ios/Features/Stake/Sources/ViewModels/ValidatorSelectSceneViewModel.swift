// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemStakeServiceProtocol
import Components
import protocol Gemstone.GemExplorerServiceProtocol
import GemstonePrimitives
import Foundation
import Localization
import Primitives
import PrimitivesComponents

@Observable
public final class ValidatorSelectSceneViewModel {
    private let type: ValidatorSelectType
    private let chain: Chain
    public let currentValidator: DelegationValidator?
    private let validators: [DelegationValidator]
    public var selectValidator: ((DelegationValidator) -> Void)?
    private let explorerService: any GemExplorerServiceProtocol


    private let stakeService: any GemStakeServiceProtocol

    public init(
        explorerService: any GemExplorerServiceProtocol,
        stakeService: any GemStakeServiceProtocol,
        type: ValidatorSelectType,
        chain: Chain,
        currentValidator: DelegationValidator?,
        validators: [DelegationValidator],
        selectValidator: ((DelegationValidator) -> Void)? = nil,
    ) {
        self.explorerService = explorerService
        self.stakeService = stakeService
        self.type = type
        self.chain = chain
        self.currentValidator = currentValidator
        self.validators = validators
        self.selectValidator = selectValidator
    }

    public var title: String {
        Localized.Stake.validators
    }

    public var list: [ListItemValueSection<DelegationValidator>] {
        switch type {
        case .stake:
            let recommended = Set(stakeService.recommendedValidatorIds(chain: chain.rawValue))
            return [
                listSection(
                    title: Localized.Common.recommended,
                    validators: validators.filter { recommended.contains($0.id) },
                ),
                listSection(
                    title: Localized.Stake.active,
                    validators: validators,
                ),
            ].filter(\.values.isNotEmpty)
        case .unstake:
            return [
                listSection(
                    title: Localized.Stake.active,
                    validators: validators,
                ),
            ]
        }
    }

    public func explorerLink(for validator: DelegationValidator) -> BlockExplorerLink? {
        explorerService.getValidatorUrl(chain: validator.chain.rawValue, address: validator.id).map { BlockExplorerLink($0) }
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
