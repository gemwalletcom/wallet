// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.ChainConfig
import class Gemstone.Config
import class Gemstone.GemAddressService
import class Gemstone.GemApplicationMetadataService
import class Gemstone.GemAssetConfigService
import class Gemstone.GemChainService
import enum Gemstone.DocsUrl
import enum Gemstone.NodeRegion
import class Gemstone.PriceAlertFormatter
import enum Gemstone.PublicUrl
import enum Gemstone.RewardsUrl
import enum Gemstone.SocialUrl
import struct Gemstone.StakeChainConfig
import struct Gemstone.SwapConfig
import typealias Gemstone.WalletConnectConfig
import Primitives

public extension GemAddressService {
    static let shared = GemAddressService()
}

public extension GemApplicationMetadataService {
    static let shared = GemApplicationMetadataService()
}

public extension GemAssetConfigService {
    static let shared = GemAssetConfigService()
}

public extension GemChainService {
    static let shared = GemChainService()
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
    public static let regions = Config.shared.getNodeRegions()

    public static func url(chain: Primitives.Chain, region: NodeRegion) -> URL {
        URL(string: Config.shared.getNodeUrl(chain: chain.rawValue, region: region))!
    }

    public static func region(url: String) -> NodeRegion? {
        Config.shared.getNodeRegion(url: url)
    }

    public static func flag(region: NodeRegion) -> String {
        Config.shared.getNodeRegionFlag(region: region)
    }

    public static func priority(region: NodeRegion) -> Int32 {
        Config.shared.getNodeRegionPriority(region: region)
    }
}

public enum AppUrl {
    private static let utmSource = "gemwallet_ios"

    public static func docs(_ item: DocsUrl) -> URL {
        URL(string: item.url())!
            .withUTM(source: utmSource)
    }

    public static func page(_ item: PublicUrl) -> URL {
        URL(string: item.url())!
            .withUTM(source: utmSource)
    }

    public static func rewards(_ item: RewardsUrl) -> URL {
        let locale = Locale.current.identifier
        return URL(string: item.url(locale: locale))!
            .withUTM(source: utmSource)
    }

    public static func social(_ item: SocialUrl) -> URL? {
        guard let socialUrl = item.url(),
              let url = URL(string: socialUrl) else { return nil }
        return url
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
