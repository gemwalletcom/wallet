package com.gemwallet.android.model

import com.wallet.core.primitives.Asset
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.BalanceMetadata
import uniffi.gemstone.GemAssetBalance
import java.math.BigInteger

data class AssetBalance(
    val asset: Asset,
    val balance: Balance<String> = Balance("0", "0", "0", "0", "0", "0", "0", "0", "0", "0"),
    val balanceAmount: Balance<Double> = Balance(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
    val totalAmount: Double = 0.0,
    val fiatTotalAmount: Double = 0.0,
    val metadata: BalanceMetadata? = null,
    val isActive: Boolean = true,
) {

    companion object {
        fun create(
            asset: Asset,
            available: String = "0",
            frozen: String = "0",
            locked: String = "0",
            staked: String = "0",
            pending: String = "0",
            rewards: String = "0",
            reserved: String = "0",
            withdrawable: String = "0",
            pendingUnconfirmed: String = "0",
            earn: String = "0",
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


private fun Balance<String>.createAmount(decimals: Int) = Balance(
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

fun Balance<String>.hasAvailable() = try {
    available.toBigInteger() > BigInteger.ZERO
} catch (_: Throwable) {
    false
}

fun Balance<Double>.getTotalAmount() = available + frozen + locked + staked + pending + rewards + earn

fun Balance<String>.getTotalAmount() = BigInteger(available) +
        BigInteger(frozen) +
        BigInteger(locked) +
        BigInteger(staked) +
        BigInteger(pending) +
        BigInteger(rewards) +
        BigInteger(earn)

fun AssetBalance.toGem() = GemAssetBalance(
    assetId = asset.id.toIdentifier(),
    available = BigInteger(balance.available),
    frozen = BigInteger(balance.frozen),
    locked = BigInteger(balance.locked),
    staked = BigInteger(balance.staked),
    pending = BigInteger(balance.pending),
    pendingUnconfirmed = BigInteger(balance.pendingUnconfirmed),
    rewards = BigInteger(balance.rewards),
    reserved = BigInteger(balance.reserved),
    withdrawable = BigInteger(balance.withdrawable),
    earn = BigInteger(balance.earn),
    metadata = metadata?.toJson(),
)
