package com.gemwallet.android.cases

import com.gemwallet.android.model.PushNotificationData
import com.gemwallet.android.model.PushNotificationData.Asset
import com.gemwallet.android.model.PushNotificationData.BuyAsset
import com.gemwallet.android.model.PushNotificationData.Swap
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.PushNotificationAsset
import com.wallet.core.primitives.PushNotificationReward
import com.wallet.core.primitives.PushNotificationSwapAsset
import com.wallet.core.primitives.PushNotificationTypes
import com.wallet.core.primitives.PushNotificationWalletAsset
import kotlinx.serialization.decodeFromString

fun parseNotificationData(rawType: String?, rawData: String?): PushNotificationData? {
    if (rawType.isNullOrEmpty()) {
        return null
    }
    val type = PushNotificationTypes.entries.firstOrNull { it.string == rawType } ?: return null
    return runCatching {
        when (type) {
            PushNotificationTypes.Transaction -> rawData.decodePayload<PushNotificationData.Transaction>()
            PushNotificationTypes.PriceAlert,
            PushNotificationTypes.Asset -> rawData.decodePayload<PushNotificationAsset>()?.let {
                Asset(
                    assetId = it.assetId,
                )
            }
            PushNotificationTypes.BuyAsset -> rawData.decodePayload<PushNotificationAsset>()?.let {
                BuyAsset(
                    assetId = it.assetId,
                )
            }
            PushNotificationTypes.FiatTransaction -> rawData.decodePayload<PushNotificationWalletAsset>()?.let {
                PushNotificationData.WalletAsset(
                    assetId = it.assetId,
                    walletId = it.walletId,
                )
            }
            PushNotificationTypes.SwapAsset -> rawData.decodePayload<PushNotificationSwapAsset>()?.let {
                Swap(
                    fromAssetId = it.fromAssetId,
                    toAssetId = it.toAssetId,
                )
            }
            PushNotificationTypes.Support -> PushNotificationData.Support
            PushNotificationTypes.Test -> null
            PushNotificationTypes.Rewards -> rawData.decodePayload<PushNotificationReward>()?.let {
                PushNotificationData.Reward
            }

            PushNotificationTypes.Stake -> rawData.decodePayload<PushNotificationWalletAsset>()?.let {
                PushNotificationData.Stake(assetId = it.assetId, walletId = it.walletId)
            }
        }
    }.getOrNull()
}

private inline fun <reified T> String?.decodePayload(): T? {
    return takeUnless { it.isNullOrEmpty() }?.let { jsonEncoder.decodeFromString<T>(it) }
}
