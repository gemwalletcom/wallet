package com.gemwallet.android.data.coordinators.add_asset

import com.gemwallet.android.application.add_asset.cases.GetAvailableTokenChains
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.isTokenSupported
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetAvailableTokenChainsImpl(
    private val getSession: GetSession,
) : GetAvailableTokenChains {

    override fun invoke(): Flow<List<Chain>?> {
        return getSession().mapLatest { session ->
            session?.wallet?.accounts?.map { it.chain }?.filter { it.isTokenSupported() }
        }
    }
}
