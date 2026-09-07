package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.domains.confirm.account
import com.wallet.core.primitives.AccountDataType
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.TransactionInputType
import uniffi.gemstone.GemTransferData
import java.math.BigInteger
import com.gemwallet.android.features.banner.views.BannersScene
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BannerEvent

@Composable
internal fun BannerItem(
    assetInfo: AssetInfo,
    onStake: (AssetId) -> Unit,
    onConfirm: (GemTransferData) -> Unit,
    onOpenPerpetuals: () -> Unit,
) {
    BannersScene(
        asset = assetInfo.asset,
        onClick = {
            when (it.event) {
                BannerEvent.Stake -> onStake(assetInfo.asset.id)
                BannerEvent.ActivateAsset -> {
                    val owner = assetInfo.owner ?: return@BannersScene
                    onConfirm(
                        GemTransferData(
                            inputType = TransactionInputType.account(assetInfo.asset, AccountDataType.Activate),
                            recipient = GemRecipient(owner.address),
                            value = BigInteger.ZERO,
                        )
                    )
                }

                BannerEvent.TradePerpetuals -> onOpenPerpetuals()
                else -> {}
            }
        },
        false
    )
}