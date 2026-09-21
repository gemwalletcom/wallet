package com.gemwallet.android.features.recipient.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.domains.asset.subtitleSymbol
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientHeadUIModel
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_head.NftHead
import com.wallet.core.primitives.Asset

@Composable
fun RecipientHead(asset: Asset, head: RecipientHeadUIModel) {
    when (head) {
        is RecipientHeadUIModel.Nft -> NftHead(head.nftAsset)

        RecipientHeadUIModel.Asset -> CenteredListHead(
            title = asset.name,
            subtitle = asset.subtitleSymbol,
            leading = { HeaderIcon(asset) },
        )
    }
}
