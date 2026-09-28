// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemRecentActivity
import Primitives

public extension RecentActivityData {
    init(_ activity: GemRecentActivity) {
        self.init(
            type: activity.activityType.toPrimitives(),
            assetId: activity.assetId,
            toAssetId: activity.toAssetId,
        )
    }
}
