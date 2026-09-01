package com.gemwallet.android.application.confirm.cases

import com.gemwallet.android.domains.confirm.ConfirmProperty
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemTransferData

interface BuildConfirmProperties {
    suspend operator fun invoke(
        transfer: GemTransferData,
        wallet: Wallet,
        assetsInfo: List<AssetInfo>,
        addressName: AddressName?,
    ): List<ConfirmProperty>
}
