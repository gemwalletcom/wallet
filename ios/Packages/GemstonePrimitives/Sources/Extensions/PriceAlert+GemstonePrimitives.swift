// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.priceAlertId
import Primitives

extension PriceAlert: Identifiable {
    public var id: String {
        do {
            return try priceAlertId(alert: json())
        } catch {
            preconditionFailure("Unencodable price alert: \(error)")
        }
    }
}

extension PriceAlertData: Identifiable {
    public var id: String {
        asset.id.identifier + priceAlert.id
    }
}
