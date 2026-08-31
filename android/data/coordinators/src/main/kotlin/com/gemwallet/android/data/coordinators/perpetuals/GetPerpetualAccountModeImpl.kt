package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetualAccountMode
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualAccountMode
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemPerpetualService

class GetPerpetualAccountModeImpl(
    private val perpetualService: GemPerpetualService,
) : GetPerpetualAccountMode {

    override suspend fun getPerpetualAccountMode(walletId: WalletId, address: String): PerpetualAccountMode = withContext(Dispatchers.IO) {
        perpetualService.accountMode(walletId.id, Chain.HyperCore.string, address).decodeJson()
    }
}
