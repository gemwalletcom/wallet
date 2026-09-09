package com.gemwallet.android.model

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.BalanceMetadata
import uniffi.gemstone.GemAssetBalance
import java.math.BigInteger

data class AssetBalance(
    val asset: Asset,
    val balance: Balance<BigInteger> = Balance.zero(),
    val balanceAmount: Balance<Double> = Balance(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    val totalAmount: Double = 0.0,
    val fiatTotalAmount: Double = 0.0,
    val metadata: BalanceMetadata? = null,
    val isActive: Boolean = true,
) {

    companion object {
        fun create(
            asset: Asset,
            available: BigInteger = BigInteger.ZERO,
            frozen: BigInteger = BigInteger.ZERO,
            locked: BigInteger = BigInteger.ZERO,
            staked: BigInteger = BigInteger.ZERO,
            pending: BigInteger = BigInteger.ZERO,
            rewards: BigInteger = BigInteger.ZERO,
            reserved: BigInteger = BigInteger.ZERO,
            withdrawable: BigInteger = BigInteger.ZERO,
            pendingUnconfirmed: BigInteger = BigInteger.ZERO,
            earn: BigInteger = BigInteger.ZERO,
            metadata: BalanceMetadata? = null,
            isActive: Boolean = true,
        ): AssetBalance {
            val balance = Balance(
                available = available,
                frozen = frozen,
                locked = locked,
                staked = staked,
                pending = pending,
                rewards = rewards,
                reserved = reserved,
                withdrawable = withdrawable,
                pendingUnconfirmed = pendingUnconfirmed,
                earn = earn,
            )
            val balanceAmount = balance.createAmount(asset.decimals)
            return AssetBalance(
                asset = asset,
                balance = balance,
                balanceAmount = balanceAmount,
                totalAmount = balanceAmount.getTotalAmount(),
                fiatTotalAmount = 0.0,
                metadata = metadata,
                isActive = isActive,
            )
        }
    }
}


private fun Balance<BigInteger>.createAmount(decimals: Int) = Balance(
    available = Crypto(available).value(decimals).stripTrailingZeros().toDouble(),
    frozen = Crypto(frozen).value(decimals).stripTrailingZeros().toDouble(),
    locked = Crypto(locked).value(decimals).stripTrailingZeros().toDouble(),
    staked = Crypto(staked).value(decimals).stripTrailingZeros().toDouble(),
    pending = Crypto(pending).value(decimals).stripTrailingZeros().toDouble(),
    rewards = Crypto(rewards).value(decimals).stripTrailingZeros().toDouble(),
    reserved = Crypto(reserved).value(decimals).stripTrailingZeros().toDouble(),
    withdrawable = Crypto(withdrawable).value(decimals).stripTrailingZeros().toDouble(),
    pendingUnconfirmed = Crypto(pendingUnconfirmed).value(decimals).stripTrailingZeros().toDouble(),
    earn = Crypto(earn).value(decimals).stripTrailingZeros().toDouble(),
)

fun Balance<BigInteger>.hasAvailable() = available > BigInteger.ZERO

fun Balance<Double>.getTotalAmount() = available + frozen + locked + staked + pending + rewards + earn

fun Balance<BigInteger>.getTotalAmount() = available + frozen + locked + staked + pending + rewards + earn

fun AssetBalance.toGem() = GemAssetBalance(
    assetId = asset.id.toIdentifier(),
    available = balance.available,
    frozen = balance.frozen,
    locked = balance.locked,
    staked = balance.staked,
    pending = balance.pending,
    pendingUnconfirmed = balance.pendingUnconfirmed,
    rewards = balance.rewards,
    reserved = balance.reserved,
    withdrawable = balance.withdrawable,
    earn = balance.earn,
    metadata = metadata?.toGem(),
    isActive = isActive,
)
