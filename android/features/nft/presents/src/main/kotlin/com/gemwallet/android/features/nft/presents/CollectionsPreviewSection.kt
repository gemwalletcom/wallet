package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.NftListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.NftItemTarget
import com.gemwallet.android.ui.models.target
import uniffi.gemstone.GemNftEntry

@Composable
fun CollectionsPreviewSection(collections: List<GemNftEntry>, onAction: (CollectionsPreviewAction) -> Unit) {
    Column {
        SubheaderItem(
            stringResource(R.string.nft_collections),
            onClick = { onAction(CollectionsPreviewAction.OpenCollections) },
        )
        collections.forEachIndexed { index, nft ->
            NftListItem(
                row = nft.row,
                listPosition = ListPosition.getPosition(index, collections.size),
                onClick = {
                    when (val target = nft.target) {
                        is NftItemTarget.Collection -> onAction(CollectionsPreviewAction.OpenCollection(target.id))
                        is NftItemTarget.Asset -> onAction(CollectionsPreviewAction.OpenAsset(target.id))
                    }
                },
            )
        }
    }
}
