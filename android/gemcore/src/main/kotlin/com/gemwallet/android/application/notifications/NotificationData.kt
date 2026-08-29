package com.gemwallet.android.application.notifications

import com.gemwallet.android.model.PushNotificationData
import com.gemwallet.android.model.PushNotificationData.Asset
import com.gemwallet.android.model.PushNotificationData.BuyAsset
import com.gemwallet.android.model.PushNotificationData.Swap
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.PushNotificationAsset
import com.wallet.core.primitives.PushNotificationReward
import com.wallet.core.primitives.PushNotificationSwapAsset
import com.wallet.core.primitives.PushNotificationTypes
import com.wallet.core.primitives.PushNotificationWalletAsset

fun parseNotificationData(rawType: String?, rawData: String?): PushNotificationData? {
    if (rawType.isNullOrEmpty()) {
        return null
    }
    val type = PushNotificationTypes.entries.firstOrNull { it.string == rawType } ?: return null
    return runCatching {
        when (type) {
            PushNotificationTypes.Transaction -> rawData?.decodeJson<PushNotificationData.Transaction>()
            PushNotificationTypes.PriceAlert,
            PushNotificationTypes.Asset -> rawData?.decodeJson<PushNotificationAsset>()?.let {
                Asset(
                    assetId = it.assetId,
                )
            }
            PushNotificationTypes.BuyAsset -> rawData?.decodeJson<PushNotificationAsset>()?.let {
                BuyAsset(
                    assetId = it.assetId,
                )
            }
            PushNotificationTypes.FiatTransaction -> rawData?.decodeJson<PushNotificationWalletAsset>()?.let {
                PushNotificationData.WalletAsset(
                    assetId = it.assetId,
                    walletId = it.walletId,
                )
            }
            PushNotificationTypes.SwapAsset -> rawData?.decodeJson<PushNotificationSwapAsset>()?.let {
                Swap(
                    fromAssetId = it.fromAssetId,
                    toAssetId = it.toAssetId,
                )
            }
            PushNotificationTypes.Support -> PushNotificationData.Support
            PushNotificationTypes.Test -> null
            PushNotificationTypes.Rewards -> rawData?.decodeJson<PushNotificationReward>()?.let {
                PushNotificationData.Reward
            }

            PushNotificationTypes.Stake -> rawData?.decodeJson<PushNotificationWalletAsset>()?.let {
                PushNotificationData.Stake(assetId = it.assetId, walletId = it.walletId)
            }
        }
    }.getOrNull()
}
