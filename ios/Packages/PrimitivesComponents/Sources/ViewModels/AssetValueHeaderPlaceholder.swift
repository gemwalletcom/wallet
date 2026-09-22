// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct AssetValueHeaderPlaceholder: ValueHeaderViewModel {
    public let assetImage: AssetImage?
    public let isWatchWallet = false
    public let title = ""
    public let subtitle: String? = nil
    public let subtitleColor = Colors.gray
    public let buttons: [HeaderButton] = []

    public init(assetImage: AssetImage) {
        self.assetImage = assetImage
    }
}
