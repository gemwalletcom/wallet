package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.nft.coordinators.GetNftCollections
import com.gemwallet.android.cases.nft.GetListNftCase
import com.wallet.core.primitives.NFTData
import kotlinx.coroutines.flow.Flow

class GetNftCollectionsImpl(
    private val getListNftCase: GetListNftCase,
) : GetNftCollections {

    override fun invoke(collectionId: String?): Flow<List<NFTData>> {
        return getListNftCase.getListNft(collectionId)
    }
}
