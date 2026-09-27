package com.gemwallet.android.data.services.store.database.entities

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockNftAssetId
import com.gemwallet.android.testkit.mockNftCollectionId
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.NFTType

fun mockDbNftAsset(id: NFTAssetId = mockNftAssetId(), collectionId: NFTCollectionId = mockNftCollectionId()) = DbNFTAsset(
    id = id,
    collectionId = collectionId,
    tokenId = id.tokenId,
    tokenType = NFTType.ERC721,
    name = id.toIdentifier(),
    chain = id.chain,
    contractAddress = id.contractAddress,
    imageUrl = "",
)
