// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.ChainConfig
import class Gemstone.Config
import enum Gemstone.DocsUrl
import enum Gemstone.PublicUrl
import enum Gemstone.RewardsUrl
import enum Gemstone.SocialUrl
import struct Gemstone.StakeChainConfig
import struct Gemstone.SwapConfig
import typealias Gemstone.WalletConnectConfig
import Primitives

private let utmSource = "gemwallet_ios"

public extension Config {
    static let shared = Config()

    func swapConfig() -> SwapConfig {
        getSwapConfig()
    }
}

public enum GemstoneConfig {
    public static let shared = Config()
}

public enum Docs {
    public static func url(_ item: DocsUrl) -> URL {
        URL(string: Config.shared.getDocsUrl(item: item))!
            .withUTM(source: utmSource)
    }
}

public enum RewardsUrlConfig {
    public static func url(_ item: RewardsUrl) -> URL {
        let locale = Locale.current.identifier
        return URL(string: Config.shared.getRewardsUrl(item: item, locale: locale))!
            .withUTM(source: utmSource)
    }
}

public enum PublicConstants {
    public static func url(_ item: PublicUrl) -> URL {
        URL(string: Config.shared.getPublicUrl(item: item))!
            .withUTM(source: utmSource)
    }
}

public enum Social {
    public static func url(_ item: SocialUrl) -> URL? {
        if let socialUrl = Config.shared.getSocialUrl(item: item), let url = URL(string: socialUrl) {
            return url
        }
        return .none
    }
}

public enum StakeConfig {
    public static func config(chain: StakeChain) -> StakeChainConfig {
        Config.shared.getStakeConfig(chain: chain.rawValue)
    }
}

public enum ChainConfig {
    // store in memory for fast access
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
