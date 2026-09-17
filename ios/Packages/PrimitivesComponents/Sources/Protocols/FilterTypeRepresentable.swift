// Copyright (c). Gem Wallet. All rights reserved.

import Components
import SwiftUI

public protocol FilterTypeRepresentable {
    var value: String { get }
    var title: String { get }
    var image: AssetImage { get }
}

public extension FilterTypeRepresentable {
    var listItem: ListItemModel {
        ListItemModel(title: title, subtitle: value, imageStyle: .settings(assetImage: image))
    }
}
