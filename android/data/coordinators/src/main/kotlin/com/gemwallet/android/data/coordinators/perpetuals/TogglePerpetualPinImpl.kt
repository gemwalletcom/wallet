package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.TogglePerpetualPin
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.wallet.core.primitives.PerpetualId
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.launch
import javax.inject.Inject

class TogglePerpetualPinImpl @Inject constructor(
    private val perpetualRepository: PerpetualRepository,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : TogglePerpetualPin {
    override fun togglePin(perpetualId: PerpetualId) {
        scope.launch {
            val current = perpetualRepository.getPerpetual(perpetualId).firstOrNull() ?: return@launch
            perpetualRepository.setPinned(perpetualId, !current.metadata.isPinned)
        }
    }
}
