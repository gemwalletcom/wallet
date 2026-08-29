package com.gemwallet.android.application.assets.cases

import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.Currency

interface GetWidgetAssets {
    suspend operator fun invoke(currency: Currency): List<AssetInfo>
}
