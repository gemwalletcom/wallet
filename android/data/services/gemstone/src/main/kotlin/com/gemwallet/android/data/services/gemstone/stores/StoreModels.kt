package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.BalanceMetadata
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.AssetPrice
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
    metadata = BalanceMetadata(
        votes = votes.toUInt(),
        energyAvailable = energyAvailable.toUInt(),
        energyTotal = energyTotal.toUInt(),
        bandwidthAvailable = bandwidthAvailable.toUInt(),
        bandwidthTotal = bandwidthTotal.toUInt(),
    ).toJson(),
)

fun DbPrice.toAssetPrice(): AssetPrice = AssetPrice(
    assetId = assetId,
    price = value ?: 0.0,
    priceChangePercentage24h = dayChanged ?: 0.0,
    updatedAt = updatedAt ?: 0L,
)
