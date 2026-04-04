package com.gemwallet.android.services

import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.data.repositories.buy.BuyRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject

class SyncService @Inject constructor(
    private val buyRepository: BuyRepository,
    private val syncDeviceInfo: SyncDeviceInfo,
) {

    suspend fun sync() = withContext(Dispatchers.IO) {
        buyRepository.sync()
        syncDeviceInfo.syncDeviceInfo()
    }
}
