// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct BannerInfo: Codable, FetchableRecord {
    let banner: BannerRecord
    let asset: AssetRecord?
}

extension BannerInfo {
    func mapToBanner() -> Banner {
        Banner(
            walletId: banner.walletId.flatMap { try? WalletId.from(id: $0) },
            asset: asset?.mapToAsset(),
            event: banner.event,
            state: banner.state,
        )
    }
}
