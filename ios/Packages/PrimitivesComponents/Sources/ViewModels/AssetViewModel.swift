import Components
import Foundation
import func Gemstone.assetText
import struct Gemstone.GemAssetText
import GemstonePrimitives
import Primitives

public struct AssetViewModel: Sendable, Identifiable, AssetPreviewable {
    public let asset: Asset
    private let text: GemAssetText

    public init(asset: Asset) {
        self.asset = asset
        text = assetText(asset: asset.toGem())
    }

    public var id: String {
        asset.id.identifier
    }

    public var title: String {
        text.title
    }

    public var name: String {
        asset.name
    }

    public var symbol: String {
        asset.symbol
    }

    public var subtitleSymbol: String? {
        text.subtitleSymbol
    }

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: asset.id).assetImage
    }

    public var networkAssetImage: AssetImage {
        AssetIdViewModel(assetId: asset.id).networkAssetImage
    }

    public var networkName: String {
        text.networkName
    }

    public var networkFullName: String {
        text.networkFullName
    }
}
