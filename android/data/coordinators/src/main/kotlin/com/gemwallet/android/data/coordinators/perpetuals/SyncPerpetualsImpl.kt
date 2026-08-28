package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals
import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import javax.inject.Inject
import uniffi.gemstone.GemPerpetualService

class SyncPerpetualsImpl @Inject constructor(
    private val perpetualService: GemPerpetualService,
) : SyncPerpetuals {
    override suspend fun syncPerpetuals() {
        runCatchingCancellable { perpetualService.syncEnablement(null) }
            .onFailure { Log.e("SyncPerpetuals", "perpetual markets sync failed", it) }
    }
}
