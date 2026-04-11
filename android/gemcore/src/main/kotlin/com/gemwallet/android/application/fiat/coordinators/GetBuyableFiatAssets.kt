package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.FiatAssets

interface GetBuyableFiatAssets {
    suspend fun getBuyableFiatAssets(): FiatAssets
}
