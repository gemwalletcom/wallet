package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import kotlinx.coroutines.flow.StateFlow
import java.math.BigInteger

sealed interface AmountDataProvider {
    val title: AmountTitle
    val canChangeValue: Boolean
    val showsAssetBalance: Boolean get() = canChangeValue
    val canSwitchInputType: Boolean
    val reserveForFee: BigInteger

    val assetInfo: StateFlow<AssetInfo?>
    val availableBalance: StateFlow<BigInteger>
    val minimumValue: StateFlow<BigInteger>

    fun shouldReserveFee(isMaxAmount: Boolean): Boolean
    suspend fun buildConfirmParams(amount: Crypto, isMax: Boolean): ConfirmParams
}
