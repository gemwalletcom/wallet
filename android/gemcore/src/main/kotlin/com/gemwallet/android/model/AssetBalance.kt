package com.gemwallet.android.model

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BalanceMetadata
import uniffi.gemstone.GemAssetBalance

data class AssetBalance(val asset: Asset, val balance: Balance = Balance.zero(), val metadata: BalanceMetadata? = null, val isActive: Boolean = true)

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
