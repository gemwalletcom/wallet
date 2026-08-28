package com.gemwallet.android.data.coordinators.session

import com.gemwallet.android.application.session.cases.SetCurrentCurrency
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemDeviceService

class SetCurrentCurrencyImpl(
    private val sessionRepository: SessionRepository,
    private val priceService: GemPriceService,
    private val deviceService: GemDeviceService,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : SetCurrentCurrency {

    override fun setCurrentCurrency(currency: Currency) {
        scope.launch {
            if (sessionRepository.getCurrentCurrency() == currency) {
                return@launch
            }

            sessionRepository.setCurrency(currency)
            priceService.changeCurrency(currency.toJson())
            deviceService.synchronizeIfNeeded()
        }
    }
}
