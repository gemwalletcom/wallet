package com.gemwallet.android.data.coordinators.add_asset

import com.gemwallet.android.application.add_asset.cases.AddCustomToken
import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.getAccount
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.flow.firstOrNull

class AddCustomTokenImpl(
    private val getSession: GetSession,
    private val enableAsset: EnableAsset,
) : AddCustomToken {

    override suspend fun invoke(chain: Chain, assetId: AssetId) {
        val session = getSession().firstOrNull() ?: return
        session.wallet.getAccount(chain) ?: return
        enableAsset(session.wallet.id, assetId)
    }
}
