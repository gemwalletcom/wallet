// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.Config

public struct FiatConfig {
    private init() {}

    public static var defaultBuyAmount: Int {
        Int(Config.shared.getFiatConfig().defaultBuyAmount)
    }

    public static var defaultSellAmount: Int {
        Int(Config.shared.getFiatConfig().defaultSellAmount)
    }

    public static var minimumAmount: Int {
        Int(Config.shared.getFiatConfig().minimumAmount)
    }

    public static var maximumAmount: Int {
        Int(Config.shared.getFiatConfig().maximumAmount)
    }

    public static var randomMaxAmount: Int {
        Int(Config.shared.getFiatConfig().randomMaxAmount)
    }

    public static var suggestedAmounts: [Int] {
        Config.shared.getFiatConfig().suggestedAmounts.map(Int.init)
    }

    public static var insufficientNetworkFeeBuyAmount: Int {
        Int(Config.shared.getFiatConfig().insufficientNetworkFeeBuyAmount)
    }
}
