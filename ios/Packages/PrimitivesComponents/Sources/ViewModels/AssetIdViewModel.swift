// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import class Gemstone.GemAssetConfigService
import struct Gemstone.GemAssetIcon
import enum Gemstone.GemAssetIconImage
import GemstonePrimitives
import Primitives
import SwiftUI

public struct AssetIdViewModel: Sendable {
    private let assetId: AssetId

    public init(assetId: AssetId) {
        self.assetId = assetId
    }

    public var networkAssetImage: AssetImage {
        AssetImage(
            type: .text(.empty),
            imageURL: .none,
            placeholder: ChainImage(chain: assetId.chain).image,
            chainPlaceholder: .none,
        )
    }

    public var assetImage: AssetImage {
        AssetImage(icon: GemAssetConfigService.shared.assetIcon(assetId: assetId.identifier))
    }
}

public extension AssetImage {
    init(icon: GemAssetIcon) {
        let (imageURL, placeholder): (URL?, Image?) = switch icon.image {
        case let .local(chain): (.none, ChainImage(chain: Chain(core: chain)).image)
        case let .localToken(token): (.none, TokenImage(token: token).image)
        case let .remote(url): (URL(string: url), .none)
        }
        self.init(
            type: .text(icon.placeholder ?? .empty),
            imageURL: imageURL,
            placeholder: placeholder,
            chainPlaceholder: icon.badge.map { ChainImage(chain: Chain(core: $0)).image },
        )
    }
}
