package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.NFTImages
import com.wallet.core.primitives.NFTResource
import com.wallet.core.primitives.NFTType

fun mockNftAsset(id: NFTAssetId = mockNftAssetId(), collectionId: NFTCollectionId = mockNftCollectionId()) = NFTAsset(
    id = id,
    collectionId = collectionId,
    chain = id.chain,
    contractAddress = id.contractAddress,
    tokenId = id.tokenId,
    tokenType = NFTType.ERC721,
    name = id.toIdentifier(),
    description = null,
    resource = NFTResource("", ""),
    images = NFTImages(NFTResource("", "")),
    attributes = emptyList(),
)
