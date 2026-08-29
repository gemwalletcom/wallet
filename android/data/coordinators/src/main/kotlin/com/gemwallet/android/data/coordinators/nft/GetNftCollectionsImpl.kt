package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.nft.cases.GetNftCollections
import com.gemwallet.android.application.nft.cases.GetListNft
import com.gemwallet.android.application.session.cases.GetSession
import com.wallet.core.primitives.NFTData
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetNftCollectionsImpl(
    private val getSession: GetSession,
    private val getListNftCase: GetListNft,
) : GetNftCollections {

    override fun invoke(collectionId: String?): Flow<List<NFTData>> {
        return getSession()
            .filterNotNull()
            .distinctUntilChangedBy { it.wallet.id }
            .flatMapLatest { getListNftCase.getListNft(it.wallet.id, collectionId) }
    }
}
