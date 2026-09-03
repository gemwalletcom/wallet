package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.getReserveBalanceUrl
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.domains.confirm.account
import com.wallet.core.primitives.AccountDataType
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData
import java.math.BigInteger
import com.gemwallet.android.ui.open
import com.gemwallet.android.features.banner.views.BannersScene
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BannerEvent
import com.gemwallet.android.AppUrl
import uniffi.gemstone.DocsUrl

@Composable
internal fun BannerItem(
    assetInfo: AssetInfo,
    onStake: (AssetId) -> Unit,
    onConfirm: (GemTransferData) -> Unit,
    onOpenPerpetuals: () -> Unit,
) {
    val context = LocalContext.current
    val uriHandler = LocalUriHandler.current
    BannersScene(
        asset = assetInfo.asset,
        onClick = {
            when (it.event) {
                BannerEvent.Stake -> onStake(assetInfo.asset.id)
                BannerEvent.AccountBlockedMultiSignature ->
                    uriHandler.open(context, AppUrl.docs(DocsUrl.TronMultiSignature))

                BannerEvent.ActivateAsset -> {
                    val owner = assetInfo.owner ?: return@BannersScene
                    onConfirm(
                        GemTransferData(
                            inputType = GemTransactionInputType.account(assetInfo.asset, AccountDataType.Activate),
                            recipient = GemRecipient(owner.address),
                            value = BigInteger.ZERO.toString(),
                        )
                    )
                }

                BannerEvent.AccountActivation -> assetInfo.asset.chain
                    .getReserveBalanceUrl()?.let { uri -> uriHandler.open(context, uri) }

                BannerEvent.TradePerpetuals -> onOpenPerpetuals()
                else -> {}
            }
        },
        false
    )
}