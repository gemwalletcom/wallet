package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.SetPerpetualPinned
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.PerpetualId
import uniffi.gemstone.GemPerpetualService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class SetPerpetualPinnedImpl(
    private val perpetualService: GemPerpetualService,
) : SetPerpetualPinned {

    override suspend fun invoke(perpetualId: PerpetualId, pinned: Boolean) = withContext(Dispatchers.IO) {
        perpetualService.setPinned(perpetualId.toIdentifier(), pinned)
    }
}
