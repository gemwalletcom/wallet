// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation

public struct GridPosterViewModel: Sendable {
    public var listItem: ListItemModel {
        ListItemModel(
            title: title,
            subtitle: countText,
            imageStyle: ListItemImageStyle(assetImage: assetImage, imageSize: .image.asset, cornerRadiusType: .custom(.small)),
        )
    }

    public let assetImage: AssetImage
    public let title: String?
    public let countText: String?
    public let isVerified: Bool

    public init(
        assetImage: AssetImage,
        title: String?,
        countText: String? = nil,
        isVerified: Bool = false,
    ) {
        self.assetImage = assetImage
        self.title = title
        self.countText = countText
        self.isVerified = isVerified
    }
}
