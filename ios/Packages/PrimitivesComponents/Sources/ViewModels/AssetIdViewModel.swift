// Copyright (c). Gem Wallet. All rights reserved.

import Components
import class Gemstone.GemAssetConfigService
import enum Gemstone.GemImage
import GemstonePrimitives
import Primitives
import SwiftUI

private let assetConfig = GemAssetConfigService()

public struct AssetIdViewModel: Sendable {
    private let assetId: AssetId

    public init(assetId: AssetId) {
        self.assetId = assetId
    }

    public var networkAssetImage: AssetImage {
        AssetImage(
            type: .text(.empty),
            imageURL: .none,
            placeholder: ChainImage(chain: assetId.chain).l2Image ?? imagePlaceholder,
            chainPlaceholder: .none,
        )
    }

    public var assetImage: AssetImage {
        let defaultAssetImage = AssetImage(
            type: .text(assetId.assetType?.rawValue ?? .empty),
            imageURL: imageURL,
            placeholder: imagePlaceholder,
            chainPlaceholder: chainPlaceholder,
        )
        let iconAssetId = AssetId(core: assetConfig.iconAssetId(assetId: assetId.identifier))
        guard iconAssetId != assetId else {
            return defaultAssetImage
        }
        let iconAssetImage = AssetIdViewModel(assetId: iconAssetId).assetImage
        return AssetImage(
            type: defaultAssetImage.type,
            imageURL: iconAssetImage.imageURL,
            placeholder: iconAssetImage.placeholder,
            chainPlaceholder: chainPlaceholder,
        )
    }

    private var imageURL: URL? {
        switch assetId.type {
        case .native: .none
        case .token: GemImage.asset(assetId: assetId.identifier).imageURL
        }
    }

    private var imagePlaceholder: Image? {
        switch assetId.type {
        case .native: ChainImage(chain: assetId.chain).image
        case .token: .none
        }
    }

    private var chainPlaceholder: Image? {
        switch assetId.type {
        case .native: ChainImage(chain: assetId.chain).l2Image
        case .token: ChainImage(chain: assetId.chain).placeholder
        }
    }
}
