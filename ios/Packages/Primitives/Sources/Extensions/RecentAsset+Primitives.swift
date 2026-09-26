// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension RecentAsset: Identifiable {
    public var id: String {
        asset.id.identifier
    }
}
