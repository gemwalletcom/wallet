package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.ext.toGem
import uniffi.gemstone.AssetPrice
import uniffi.gemstone.GemAssetBalance
import java.math.BigInteger

fun DbBalance.toGemAssetBalance(): GemAssetBalance = GemAssetBalance(
    assetId = assetId,
    available = BigInteger(available),
    frozen = BigInteger(frozen),
    locked = BigInteger(locked),
    staked = BigInteger(staked),
    pending = BigInteger(pending),
    pendingUnconfirmed = BigInteger(pendingUnconfirmed),
    rewards = BigInteger(rewards),
    reserved = BigInteger(reserved),
    withdrawable = BigInteger(withdrawable),
    earn = BigInteger(earn),
    metadata = metadata?.toGem(),
    isActive = isActive,
)

fun DbPrice.toAssetPrice(): AssetPrice = AssetPrice(
    assetId = assetId,
    price = value ?: 0.0,
    priceChangePercentage24h = dayChanged ?: 0.0,
    updatedAt = updatedAt ?: 0L,
)
