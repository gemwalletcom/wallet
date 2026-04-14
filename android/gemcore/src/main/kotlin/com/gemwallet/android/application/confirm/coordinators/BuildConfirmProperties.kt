package com.gemwallet.android.application.confirm.coordinators

import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams

interface BuildConfirmProperties {
    suspend operator fun invoke(
        request: ConfirmParams,
        assetsInfo: List<AssetInfo>,
    ): List<ConfirmProperty>
}
