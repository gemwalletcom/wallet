package com.gemwallet.android.application.confirm.cases

import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Wallet

interface BuildConfirmProperties {
    suspend operator fun invoke(
        request: ConfirmParams,
        wallet: Wallet,
        assetsInfo: List<AssetInfo>,
        addressName: AddressName?,
    ): List<ConfirmProperty>
}
