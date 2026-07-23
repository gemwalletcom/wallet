package com.gemwallet.android.application.swap.coordinators

import com.wallet.core.primitives.FiatAssets

interface GetSwapAssets {
    suspend operator fun invoke(): FiatAssets
}
