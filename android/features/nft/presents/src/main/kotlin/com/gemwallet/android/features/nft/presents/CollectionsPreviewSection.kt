package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.nft.viewmodels.NftListViewModels
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.NftListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun CollectionsPreviewSection(
    onAction: (CollectionsPreviewAction) -> Unit,
    viewModel: NftListViewModels = hiltViewModel(),
) {
    val collections by viewModel.collections.collectAsStateWithLifecycle()

    Column {
        SubheaderItem(
            stringResource(R.string.nft_collections),
            onClick = { onAction(CollectionsPreviewAction.OpenCollections) },
        )
        collections.forEachIndexed { index, nft ->
            NftListItem(
                model = nft,
                listPosition = ListPosition.getPosition(index, collections.size),
                onClick = {
                    val asset = nft.asset
                    if (asset == null) {
                        onAction(CollectionsPreviewAction.OpenCollection(nft.collection.id.toIdentifier()))
                    } else {
                        onAction(CollectionsPreviewAction.OpenAsset(asset.id))
                    }
                },
            )
        }
    }
}
