// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives

struct NFTAssetRecordInfo: Codable, FetchableRecord {
    let asset: NFTAssetRecord
    let collection: NFTCollectionRecord
    let associations: [NFTAssetAssociationRecord]
}

extension NFTAssetRecordInfo {
    func mapToDetails() -> NFTAssetDetails {
        NFTAssetDetails(
            assetData: NFTAssetData(collection: collection.toNFTCollection(), asset: asset.toNFTAsset()),
            isOwned: associations.isNotEmpty,
        )
    }
}
