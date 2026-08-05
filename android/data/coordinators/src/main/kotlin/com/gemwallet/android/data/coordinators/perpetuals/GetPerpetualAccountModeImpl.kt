package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualAccountMode
import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.data.repositories.config.UserConfig
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualAccountMode
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject

class GetPerpetualAccountModeImpl @Inject constructor(
    private val perpetualService: PerpetualService,
    private val userConfig: UserConfig,
) : GetPerpetualAccountMode {

    override suspend fun getPerpetualAccountMode(walletId: WalletId, address: String): PerpetualAccountMode = withContext(Dispatchers.IO) {
        val mode = perpetualService.getAccountMode(Chain.HyperCore, address)
        userConfig.setPerpetualAccountMode(walletId, mode)
        mode
    }
}
