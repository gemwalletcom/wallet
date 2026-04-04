package com.gemwallet.android.data.coordinators.session

import com.gemwallet.android.application.session.coordinators.SetCurrentCurrency
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

class SetCurrentCurrencyImpl(
    private val sessionRepository: SessionRepository,
    private val syncDeviceInfo: SyncDeviceInfo,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : SetCurrentCurrency {

    override fun setCurrentCurrency(currency: Currency) {
        scope.launch {
            if (sessionRepository.getCurrentCurrency() == currency) {
                return@launch
            }

            sessionRepository.setCurrency(currency)
            syncDeviceInfo.syncDeviceInfo()
        }
    }
}
