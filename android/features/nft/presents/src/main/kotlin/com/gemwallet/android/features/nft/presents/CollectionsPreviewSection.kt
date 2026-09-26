package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.nft.viewmodels.CollectionsViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.NftListItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.NftItemTarget

@Composable
fun CollectionsPreviewSection(onAction: (CollectionsPreviewAction) -> Unit, viewModel: CollectionsViewModel = hiltViewModel()) {
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
                    when (val target = nft.target) {
                        is NftItemTarget.Collection -> onAction(CollectionsPreviewAction.OpenCollection(target.id))
                        is NftItemTarget.Asset -> onAction(CollectionsPreviewAction.OpenAsset(target.id))
                    }
                },
            )
        }
    }
}
