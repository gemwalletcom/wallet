// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import SwiftUI

struct InsightLink {
    var listItem: ListItemModel {
        ListItemModel(title: title, subtitle: subtitle, imageStyle: .settings(assetImage: image))
    }

    let title: String
    let subtitle: String?
    var url: URL
    let deepLink: URL?
    let image: AssetImage
}

extension InsightLink: Identifiable {
    var id: String {
        title + url.absoluteString
    }
}
