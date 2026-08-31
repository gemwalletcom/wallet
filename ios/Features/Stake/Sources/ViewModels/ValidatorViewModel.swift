// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ValidatorViewModel {
    public let validator: DelegationValidator
    private let imageFormatter = AssetImageFormatter()

    public init(validator: DelegationValidator) {
        self.validator = validator
    }

    public var name: String {
        switch validator.providerType {
        case .stake:
            if validator.name.isEmpty {
                return AddressFormatter(style: .short, address: validator.id, chain: validator.chain).value()
            }
            return validator.name
        case .earn:
            return validator.name
        }
    }

    public var aprModel: AprViewModel {
        AprViewModel(apr: validator.apr)
    }

    public var imageUrl: URL? {
        switch validator.providerType {
        case .stake:
            validator.id == DelegationValidator.systemId
                ? imageFormatter.getURL(for: validator.chain.assetId)
                : imageFormatter.getValidatorUrl(chain: validator.chain, id: validator.id)
        case .earn:
            nil
        }
    }

    public var image: Image? {
        switch validator.providerType {
        case .stake:
            nil
        case .earn:
            switch YieldProvider(rawValue: validator.id) {
            case .yo: Images.EarnProviders.yo
            case .none: nil
            }
        }
    }

    public var validatorImage: AssetImage {
        switch validator.providerType {
        case .stake:
            AssetImage(
                type: .text(String(name.first ?? " ")),
                imageURL: imageUrl,
            )
        case .earn:
            AssetImage(placeholder: image)
        }
    }
}
