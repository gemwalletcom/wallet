package com.gemwallet.android.data.coordinators.add_asset

import com.gemwallet.android.application.add_asset.coordinators.GetAvailableTokenChains
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.assetType
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetAvailableTokenChainsImpl(
    private val sessionRepository: SessionRepository,
) : GetAvailableTokenChains {

    override fun invoke(): Flow<List<Chain>?> {
        return sessionRepository.session().mapLatest { session ->
            session?.wallet?.accounts?.map { it.chain }?.filter { it.assetType() != null }
        }
    }
}
