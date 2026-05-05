package com.gemwallet.android.data.coordinators.add_asset

import com.gemwallet.android.application.add_asset.coordinators.AddCustomToken
import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.firstOrNull

class AddCustomTokenImpl(
    private val sessionRepository: SessionRepository,
    private val enableAsset: EnableAsset,
) : AddCustomToken {

    override suspend fun invoke(chain: Chain, assetId: AssetId) {
        val session = sessionRepository.session().firstOrNull() ?: return
        session.wallet.getAccount(chain) ?: return
        enableAsset(session.wallet.id, assetId)
    }
}
