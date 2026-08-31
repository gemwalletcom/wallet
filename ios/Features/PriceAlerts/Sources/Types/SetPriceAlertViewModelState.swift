// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

struct SetPriceAlertViewModelState {
    var type: SetPriceAlertType = .price
    var selectedDirection: PriceAlertDirection = .up

    var amount: String {
        get {
            switch type {
            case .price: priceAmount
            case .percentage: percentageAmount
            }
        }
        set {
            switch type {
            case .price: priceAmount = newValue
            case .percentage: percentageAmount = newValue
            }
        }
    }

    private var priceAmount: String = .empty
    private var percentageAmount: String = .empty

    init(price: Double? = nil) {
        if let price {
            type = .price
            priceAmount = String(price)
        }
    }
}
