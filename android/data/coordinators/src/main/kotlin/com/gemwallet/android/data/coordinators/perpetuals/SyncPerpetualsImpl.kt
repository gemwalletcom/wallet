package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.SyncPerpetuals
import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import javax.inject.Inject
import uniffi.gemstone.GemMarketsRefreshTrigger
import uniffi.gemstone.GemPerpetualServiceInterface

class SyncPerpetualsImpl @Inject constructor(
    private val perpetualService: GemPerpetualServiceInterface,
) : SyncPerpetuals {
    override suspend fun syncPerpetuals(trigger: GemMarketsRefreshTrigger) {
        runCatchingCancellable { perpetualService.syncEnablement(null, trigger) }
            .onFailure { Log.e("SyncPerpetuals", "perpetual markets sync failed", it) }
    }
}
