package com.gemwallet.android.testkit

import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.Balance
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BalanceMetadata
import java.math.BigInteger

fun mockAssetBalance(
    asset: Asset = mockAsset(),
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
    return AssetBalance(
        asset = asset,
        balance = balance,
        metadata = metadata,
        isActive = isActive,
    )
}
