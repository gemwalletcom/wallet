package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.FiatAssets

interface GetSellableFiatAssets {
    suspend fun getSellableFiatAssets(): FiatAssets
}
