// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import SwiftUI

extension ListItemImageStyle {
    static func security(_ emoji: String) -> ListItemImageStyle? {
        ListItemImageStyle(
            assetImage: AssetImage(type: emoji),
            imageSize: .image.semiMedium,
            alignment: .top,
            cornerRadiusType: .none,
        )
    }
}
