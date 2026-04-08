package com.gemwallet.android.application.nft.coordinators

import com.wallet.core.primitives.NFTData
import kotlinx.coroutines.flow.Flow

interface GetNftCollections {
    fun getNftCollections(collectionId: String?): Flow<List<NFTData>>
}
