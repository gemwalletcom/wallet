// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.priceAlertId
import Primitives

extension PriceAlert: @retroactive Identifiable {
    public var id: String {
        do {
            return try priceAlertId(alert: json())
        } catch {
            preconditionFailure("Unencodable price alert: \(error)")
        }
    }
}

extension PriceAlertData: @retroactive Identifiable {
    public var id: String {
        asset.id.identifier + priceAlert.id
    }
}
