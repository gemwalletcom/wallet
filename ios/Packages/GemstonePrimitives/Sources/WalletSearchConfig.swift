// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.Config

public struct WalletSearchConfig {
    private init() {}

    public static var assetsInitialLimit: Int {
        Int(Config.shared.getWalletSearchConfig().assetsInitialLimit)
    }

    public static var assetsTagLimit: Int {
        Int(Config.shared.getWalletSearchConfig().assetsTagLimit)
    }

    public static var assetsSearchLimit: Int {
        Int(Config.shared.getWalletSearchConfig().assetsSearchLimit)
    }

    public static var perpetualsPreviewLimit: Int {
        Int(Config.shared.getWalletSearchConfig().perpetualsPreviewLimit)
    }

    public static var resultsLimit: Int {
        Int(Config.shared.getWalletSearchConfig().resultsLimit)
    }
}
