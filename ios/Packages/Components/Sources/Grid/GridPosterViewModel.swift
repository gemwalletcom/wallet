// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation

public struct GridPosterViewModel: Sendable {
    public var listItem: ListItemModel {
        ListItemModel(
            title: title,
            subtitle: count.map { String($0) },
            imageStyle: ListItemImageStyle(assetImage: assetImage, imageSize: .image.asset, cornerRadiusType: .custom(.small)),
        )
    }

    public let assetImage: AssetImage
    public let title: String?
    public let count: Int?
    public let isVerified: Bool

    public init(
        assetImage: AssetImage,
        title: String?,
        count: Int? = nil,
        isVerified: Bool = false,
    ) {
        self.assetImage = assetImage
        self.title = title
        self.count = count
        self.isVerified = isVerified
    }
}
