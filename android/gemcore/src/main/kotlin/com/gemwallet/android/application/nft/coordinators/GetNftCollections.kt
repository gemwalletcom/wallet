package com.gemwallet.android.application.nft.coordinators

import com.wallet.core.primitives.NFTData
import kotlinx.coroutines.flow.Flow

interface GetNftCollections {
    operator fun invoke(collectionId: String?): Flow<List<NFTData>>
}
