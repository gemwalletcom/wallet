// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct ChainViewModel: Sendable {
    private let chain: Chain
    private let standard: String?

    public init(chain: Chain, standard: String? = nil) {
        self.chain = chain
        self.standard = standard
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

// MARK: - List Item

public extension ChainViewModel {
    var listItem: ListItemModel {
        ListItemModel(
            title: title,
            titleStyle: standard == nil ? .body : .body.weight(.medium),
            titleExtra: standard,
            titleStyleExtra: .calloutSecondary,
            imageStyle: .asset(assetImage: AssetImage.image(image)),
        )
    }
}
