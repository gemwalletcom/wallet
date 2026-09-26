// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct NFTCollectionRecordInfo: Codable, FetchableRecord {
    let collection: NFTCollectionRecord
    let assets: [NFTAssetRecord]
}

extension NFTCollectionRecordInfo {
    func mapToNFTData() -> NFTData {
        NFTData(
            collection: collection.toNFTCollection(),
            assets: assets.map { $0.toNFTAsset() },
        )
    }
}
