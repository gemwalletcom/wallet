package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.ext.millisToSeconds
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.BalanceMetadata
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemAssetPrice

fun DbBalance.toGemAssetBalance(): GemAssetBalance = GemAssetBalance(
    assetId = assetId,
    available = available,
    frozen = frozen,
    locked = locked,
    staked = staked,
    pending = pending,
    pendingUnconfirmed = pendingUnconfirmed,
    rewards = rewards,
    reserved = reserved,
    withdrawable = withdrawable,
    earn = earn,
    metadata = BalanceMetadata(
        votes = votes.toUInt(),
        energyAvailable = energyAvailable.toUInt(),
        energyTotal = energyTotal.toUInt(),
        bandwidthAvailable = bandwidthAvailable.toUInt(),
        bandwidthTotal = bandwidthTotal.toUInt(),
    ).toJson(),
)

fun DbPrice.toGemAssetPrice(): GemAssetPrice = GemAssetPrice(
    assetId = assetId,
    price = value ?: 0.0,
    priceChangePercentage24h = dayChanged ?: 0.0,
    updatedAt = (updatedAt ?: 0L).millisToSeconds(),
)
