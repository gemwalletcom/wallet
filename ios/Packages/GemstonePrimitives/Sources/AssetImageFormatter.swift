// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.ImageFormatter
import Primitives

public struct AssetImageFormatter: Sendable {
    public static let shared = AssetImageFormatter()

    private let formatter = ImageFormatter()

    public init() {}

    public func getURL(for assetId: AssetId) -> URL {
        URL(string: formatter.getAssetUrl(chain: assetId.chain.rawValue, tokenId: assetId.tokenId))!
    }

    public func getNFTUrl(for assetId: String) -> URL {
        URL(string: formatter.getNftAssetUrl(id: assetId))!
    }

    public func getValidatorUrl(chain: Chain, id: String) -> URL {
        URL(string: formatter.getValidatorUrl(chain: chain.rawValue, id: id))!
    }

    public func getListUrl(for id: String) -> URL {
        URL(string: formatter.getListUrl(id: id))!
    }
}
