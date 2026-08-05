package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualAccountMode
import com.gemwallet.android.blockchain.services.PerpetualService
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualAccountMode
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject

class GetPerpetualAccountModeImpl @Inject constructor(
    private val perpetualService: PerpetualService,
) : GetPerpetualAccountMode {

    override suspend fun getPerpetualAccountMode(address: String): PerpetualAccountMode = withContext(Dispatchers.IO) {
        perpetualService.getAccountMode(Chain.HyperCore, address)
    }
}
