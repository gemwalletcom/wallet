package com.gemwallet.android.ui.models.actions

import com.gemwallet.android.application.confirm.coordinators.ConfirmTransaction.FinishRoute
import com.wallet.core.primitives.AssetId

fun interface FinishConfirmAction {
    operator fun invoke(assetId: AssetId, hash: String, route: FinishRoute)
}
