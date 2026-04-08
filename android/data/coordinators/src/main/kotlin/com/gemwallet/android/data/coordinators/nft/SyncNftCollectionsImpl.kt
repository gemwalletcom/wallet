package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.nft.coordinators.SyncNftCollections
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.data.repositories.session.SessionRepository
import kotlinx.coroutines.flow.firstOrNull

class SyncNftCollectionsImpl(
    private val sessionRepository: SessionRepository,
    private val syncNfts: SyncNfts,
) : SyncNftCollections {

    override suspend fun invoke() {
        val wallet = sessionRepository.session().firstOrNull()?.wallet ?: return
        syncNfts.syncNfts(wallet)
    }
}
