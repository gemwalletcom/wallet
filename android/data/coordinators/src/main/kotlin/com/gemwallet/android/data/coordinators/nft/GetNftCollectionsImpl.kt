package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.nft.coordinators.GetNftCollections
import com.gemwallet.android.cases.nft.GetListNftCase
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.NFTData
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetNftCollectionsImpl(
    private val sessionRepository: SessionRepository,
    private val getListNftCase: GetListNftCase,
) : GetNftCollections {

    override fun invoke(collectionId: String?): Flow<List<NFTData>> {
        return sessionRepository.session()
            .filterNotNull()
            .distinctUntilChangedBy { it.wallet.id }
            .flatMapLatest { getListNftCase.getListNft(it.wallet.id, collectionId) }
    }
}
