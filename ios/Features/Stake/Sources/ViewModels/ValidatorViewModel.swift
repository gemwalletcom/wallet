// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import class Gemstone.GemAddressService
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import enum Gemstone.GemImage
import SwiftUI

public struct ValidatorViewModel {
    public let validator: DelegationValidator
    public init(validator: DelegationValidator) {
        self.validator = validator
    }

    public var name: String {
        switch validator.providerType {
        case .stake:
            if validator.name.isEmpty {
                return GemAddressService.shared.format(address: validator.id, chain: validator.chain)
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
            GemImage.validator(chain: validator.chain.rawValue, validatorId: validator.id).imageURL
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
