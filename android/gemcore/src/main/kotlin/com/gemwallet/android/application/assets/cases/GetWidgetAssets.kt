package com.gemwallet.android.application.assets.cases

import com.gemwallet.android.model.AssetInfo

interface GetWidgetAssets {
    suspend operator fun invoke(): List<AssetInfo>
}
