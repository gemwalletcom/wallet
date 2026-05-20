// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.Config

public struct PerpetualConfig {
    private init() {}

    public static var defaultLeverage: UInt8 {
        Config.shared.getPerpetualConfig().defaultLeverage
    }

    public static var leverageOptions: [UInt8] {
        Array(Config.shared.getPerpetualConfig().leverageOptions)
    }

    public static func selectLeverage(desired: UInt8, options: [UInt8]) -> UInt8 {
        Config.shared.selectLeverage(desired: desired, options: Data(options))
    }
}
