package com.gemwallet.android.features.recipient.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientType
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_head.NftHead
import com.gemwallet.android.ui.models.subtitleSymbol

@Composable
fun RecipientHead(type: RecipientType) = when (type) {
    is RecipientType.Nft -> NftHead(type.nftAsset)
    is RecipientType.Asset -> type.assetInfo.asset.let { asset ->
        CenteredListHead(
            title = asset.name,
            subtitle = asset.subtitleSymbol,
            leading = { HeaderIcon(asset) },
        )
    }
}
