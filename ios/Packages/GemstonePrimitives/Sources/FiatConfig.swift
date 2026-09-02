// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.Config

public struct FiatConfig {
    private init() {}

    public static var insufficientNetworkFeeBuyAmount: Int {
        Int(Config.shared.getFiatConfig().insufficientNetworkFeeBuyAmount)
    }
}
