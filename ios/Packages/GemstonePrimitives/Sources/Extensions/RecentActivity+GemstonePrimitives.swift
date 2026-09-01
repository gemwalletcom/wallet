// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemRecentActivity
import Primitives

public extension RecentActivityData {
    init(_ activity: GemRecentActivity) {
        self.init(
            type: activity.activityType.map(),
            assetId: Primitives.AssetId(core: activity.assetId),
            toAssetId: activity.toAssetId.map { Primitives.AssetId(core: $0) },
        )
    }
}
