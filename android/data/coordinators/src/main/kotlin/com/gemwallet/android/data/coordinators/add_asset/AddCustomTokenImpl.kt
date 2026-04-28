package com.gemwallet.android.data.coordinators.add_asset

import com.gemwallet.android.application.add_asset.coordinators.AddCustomToken
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.firstOrNull

class AddCustomTokenImpl(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
) : AddCustomToken {

    override suspend fun invoke(chain: Chain, assetId: AssetId) {
        val session = sessionRepository.session().firstOrNull() ?: return
        session.wallet.getAccount(chain) ?: return
        assetsRepository.switchVisibility(
            walletId = session.wallet.id,
            assetId = assetId,
            visibility = true,
        )
    }
}
