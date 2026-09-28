// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.ChainConfig
import class Gemstone.Config
import enum Gemstone.DocsUrl
import class Gemstone.GemAddressService
import class Gemstone.GemAssetConfigService
import class Gemstone.GemChainService
import class Gemstone.GemConnectionService
import enum Gemstone.NodeRegion
import class Gemstone.PriceAlertFormatter
import enum Gemstone.PublicUrl
import enum Gemstone.RewardsUrl
import struct Gemstone.SwapConfig
import typealias Gemstone.WalletConnectConfig
import Primitives

public extension GemAddressService {
    static let shared = GemAddressService()
}

public extension GemAssetConfigService {
    static let shared = GemAssetConfigService()
}

public extension GemChainService {
    static let shared = GemChainService()
}

public extension GemConnectionService {
    static let shared = GemConnectionService()
}

public extension PriceAlertFormatter {
    static let shared = PriceAlertFormatter()
}

public extension Config {
    static let shared = Config()

    func swapConfig() -> SwapConfig {
        getSwapConfig()
    }
}

public enum NodeURL {
    public static func url(chain: Primitives.Chain, region: NodeRegion) -> URL {
        URL(string: Config.shared.getNodeUrl(chain: chain.rawValue, region: region))!
    }
}

public enum AppUrl {
    public static func docs(_ item: DocsUrl) -> URL {
        URL(string: item.urlFor(platform: .ios))!
    }

    public static func page(_ item: PublicUrl) -> URL {
        URL(string: item.urlFor(platform: .ios))!
    }

    public static func rewards(_ item: RewardsUrl) -> URL {
        URL(string: item.urlFor(locale: Locale.current.identifier, platform: .ios))!
    }
}

public enum ChainConfig {
    /// store in memory for fast access
    private static let chainConfigs: [Primitives.Chain: Gemstone.ChainConfig] = Primitives.Chain.allCases.reduce(into: [:]) { result, chain in
        result[chain] = Config.shared.getChainConfig(chain: chain.rawValue)
    }

    public static func config(chain: Primitives.Chain) -> Gemstone.ChainConfig {
        chainConfigs[chain]!
    }
}

public enum WalletConnectConfig {
    public static func config() -> Gemstone.WalletConnectConfig {
        Config.shared.getWalletConnectConfig()
    }
}
