package com.gemwallet.android.application.fiat.coordinators

import com.wallet.core.primitives.AssetId

interface AddBuyRecent {
    suspend operator fun invoke(assetId: AssetId, walletId: String)
}
