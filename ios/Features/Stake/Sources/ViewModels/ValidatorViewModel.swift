// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemValidatorRow
import enum Gemstone.YieldProvider
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ValidatorViewModel: Sendable {
    public let row: GemValidatorRow

    public init(row: GemValidatorRow) {
        self.row = row
    }

    public var validator: DelegationValidator {
        row.validator.toPrimitives()
    }

    public var listItem: ListItemModel {
        ListItemModel(title: name, subtitle: aprText)
    }

    public var name: String {
        row.name
    }

    public var aprText: String {
        Localized.Stake.apr(row.apr?.text() ?? .empty)
    }

    public var providerImage: Image? {
        switch row.provider {
        case .yo: Images.EarnProviders.yo
        case .none: nil
        }
    }

    public var validatorImage: AssetImage {
        if let providerImage {
            return AssetImage(placeholder: providerImage)
        }
        return AssetImage(type: .text(row.placeholder), imageURL: row.imageUrl.asURL)
    }
}
