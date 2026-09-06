// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct ChainViewModel: Sendable {
    private let chain: Chain
    private let assetType: AssetType?

    public init(chain: Chain, assetType: AssetType? = nil) {
        self.chain = chain
        self.assetType = assetType
    }

    public var title: String {
        chain.networkName
    }

    public var image: Image {
        ChainImage(chain: chain).image
    }
}

// MARK: - Identifiable

extension ChainViewModel: Identifiable {
    public var id: String {
        chain.rawValue
    }
}

// MARK: - SimpleListItemViewable

extension ChainViewModel: SimpleListItemViewable {
    public var titleStyle: TextStyle {
        assetType == nil
            ? .body
            : .body.weight(.medium)
    }

    public var titleExtra: String? {
        assetType?.rawValue
    }

    public var titleStyleExtra: TextStyle {
        .calloutSecondary
    }

    public var assetImage: AssetImage {
        AssetImage.image(image)
    }
}
