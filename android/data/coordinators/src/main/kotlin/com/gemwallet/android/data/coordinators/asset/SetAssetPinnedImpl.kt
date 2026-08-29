package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.SetAssetPinned
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemBalanceService

class SetAssetPinnedImpl(
    private val getSession: GetSession,
    private val balanceService: GemBalanceService,
) : SetAssetPinned {

    override suspend fun invoke(assetId: AssetId, pinned: Boolean) {
        val session = getSession().value ?: return
        balanceService.setAssetPinned(
            walletId = session.wallet.id.id,
            assetId = assetId.toIdentifier(),
            pinned = pinned,
        )
    }
}
