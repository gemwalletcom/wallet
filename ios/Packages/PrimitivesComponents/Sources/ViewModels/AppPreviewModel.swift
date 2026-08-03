// Copyright (c). Gem Wallet. All rights reserved.

import Components

public struct AppPreviewModel: AssetPreviewable {
    public let assetImage: AssetImage
    public let name: String
    public let subtitleSymbol: String?

    public init(assetImage: AssetImage, name: String, subtitleSymbol: String?) {
        self.assetImage = assetImage
        self.name = name
        self.subtitleSymbol = subtitleSymbol
    }
}
