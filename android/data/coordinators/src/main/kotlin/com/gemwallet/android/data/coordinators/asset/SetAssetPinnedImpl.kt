package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.SetAssetPinned
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemBalanceService

class SetAssetPinnedImpl(
    private val sessionRepository: SessionRepository,
    private val balanceService: GemBalanceService,
) : SetAssetPinned {

    override suspend fun invoke(assetId: AssetId, pinned: Boolean) {
        val session = sessionRepository.session().value ?: return
        balanceService.setAssetPinned(
            walletId = session.wallet.id.id,
            assetId = assetId.toIdentifier(),
            pinned = pinned,
        )
    }
}
