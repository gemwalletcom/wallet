package com.gemwallet.android.features.recipient.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_head.NftHead
import com.gemwallet.android.ui.models.subtitleSymbol
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemRecipientType

@Composable
fun RecipientHead(asset: Asset, type: GemRecipientType) {
    when (type) {
        is GemRecipientType.Nft -> NftHead(type.nftAsset.toPrimitives())
        is GemRecipientType.Asset -> CenteredListHead(
            title = asset.name,
            subtitle = asset.subtitleSymbol,
            leading = { HeaderIcon(asset) },
        )
    }
}
