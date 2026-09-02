package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.serializer.toJson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemTransferBalance
import uniffi.gemstone.GemAmountLimits
import uniffi.gemstone.GemAmountRules
import uniffi.gemstone.GemAmountException
import uniffi.gemstone.GemAmountType
import java.math.BigInteger

abstract class AmountDataProvider(
    private val scope: CoroutineScope,
) {
    abstract val title: AmountTitle
    abstract val canSwitchInputType: Boolean
    abstract val assetInfo: StateFlow<AssetInfo?>
    abstract val amountType: StateFlow<GemAmountType?>

    open val balance: StateFlow<GemTransferBalance?> by lazy {
        assetInfo.map { it?.toAmountBalance() }.stateIn(scope, SharingStarted.Eagerly, null)
    }

    val rules: StateFlow<GemAmountRules?> by lazy {
        combine(amountType, assetInfo) { type, current ->
            if (type == null || current == null) null else type.rules(current.asset.toGem())
        }.stateIn(scope, SharingStarted.Eagerly, null)
    }

    val limits: StateFlow<GemAmountLimits?> by lazy {
        combine(amountType, assetInfo, balance) { type, current, currentBalance ->
            if (type == null || current == null || currentBalance == null) {
                null
            } else {
                try {
                    type.limits(current.asset.toGem(), currentBalance)
                } catch (_: GemAmountException) {
                    null
                }
            }
        }.stateIn(scope, SharingStarted.Eagerly, null)
    }

    val canChangeValue: StateFlow<Boolean> by lazy { rules.map { it?.canChangeValue ?: true }.stateIn(scope, SharingStarted.Eagerly, true) }
    val showsAssetBalance: StateFlow<Boolean> by lazy { rules.map { it?.showsAssetBalance ?: true }.stateIn(scope, SharingStarted.Eagerly, true) }
    val minimumValue: StateFlow<BigInteger> by lazy { rules.map { it?.minimumValue?.toBigIntegerOrNull() ?: BigInteger.ZERO }.stateIn(scope, SharingStarted.Eagerly, BigInteger.ZERO) }
    val reserveForFee: StateFlow<BigInteger> by lazy { rules.map { it?.reserveForFee?.toBigIntegerOrNull() ?: BigInteger.ZERO }.stateIn(scope, SharingStarted.Eagerly, BigInteger.ZERO) }
    val availableBalance: StateFlow<BigInteger> by lazy { limits.map { it?.availableValue?.toBigIntegerOrNull() ?: BigInteger.ZERO }.stateIn(scope, SharingStarted.Eagerly, BigInteger.ZERO) }

    fun maxValue(): BigInteger = limits.value?.maxValue?.toBigIntegerOrNull() ?: availableBalance.value

    abstract suspend fun buildConfirmInput(amount: Crypto, isMax: Boolean): GemConfirmInput
}

fun AssetInfo.toAmountBalance(): GemTransferBalance = GemTransferBalance(
    available = balance.balance.available,
    frozen = balance.balance.frozen,
    locked = balance.balance.locked,
    withdrawable = balance.balance.withdrawable,
    votes = balance.metadata?.votes ?: 0u,
)
