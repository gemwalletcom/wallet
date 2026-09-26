package com.gemwallet.android.features.transfer.presents.recipient.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.transfer.viewmodels.recipient.models.RecipientHeadUIModel
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_head.NftHead

@Composable
fun RecipientHead(head: RecipientHeadUIModel) {
    when (head) {
        is RecipientHeadUIModel.Nft -> NftHead(head.nftAsset)

        is RecipientHeadUIModel.Asset -> CenteredListHead(
            title = head.text.asset.name,
            subtitle = head.text.subtitleSymbol,
            leading = { HeaderIcon(head.text.icon) },
        )
    }
}
