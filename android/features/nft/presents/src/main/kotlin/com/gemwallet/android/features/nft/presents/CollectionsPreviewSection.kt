package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.nft.viewmodels.NftListViewModels
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.image.NftImage
import com.gemwallet.android.ui.components.image.toImageSource
import com.gemwallet.android.ui.components.list_item.ChevronIcon
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.ListItemDefaults
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.NFTAssetId

@Composable
fun CollectionsPreviewSection(
    onOpenCollections: () -> Unit,
    onOpenCollection: (String) -> Unit,
    onOpenAsset: (NFTAssetId) -> Unit,
    onOpenUnverified: () -> Unit,
    viewModel: NftListViewModels = hiltViewModel(),
) {
    val collections by viewModel.collections.collectAsStateWithLifecycle()
    val unverifiedCount by viewModel.unverifiedCount.collectAsStateWithLifecycle()

    val showUnverified = unverifiedCount > 0
    val total = collections.size + if (showUnverified) 1 else 0

    Column {
        SubheaderItem(stringResource(R.string.nft_collections), onClick = onOpenCollections)
        collections.forEachIndexed { index, nft ->
            CollectionRow(
                model = nft,
                listPosition = ListPosition.getPosition(index, total),
                onClick = {
                    val asset = nft.asset
                    if (asset == null) {
                        onOpenCollection(nft.collection.id.toIdentifier())
                    } else {
                        onOpenAsset(asset.id)
                    }
                },
            )
        }
        if (showUnverified) {
            ListItem(
                modifier = Modifier.clickable(onClick = onOpenUnverified),
                listPosition = ListPosition.getPosition(collections.size, total),
                minHeight = ListItemDefaults.iconMinHeight,
                title = { Text(stringResource(R.string.asset_verification_unverified)) },
                trailing = { CountChevron(unverifiedCount) },
            )
        }
    }
}

@Composable
private fun CollectionRow(
    model: NftItemUIModel,
    listPosition: ListPosition,
    onClick: () -> Unit,
) {
    ListItem(
        modifier = Modifier.clickable(onClick = onClick),
        listPosition = listPosition,
        minHeight = ListItemDefaults.iconMinHeight,
        leading = {
            NftImage(
                source = model.toImageSource(),
                modifier = Modifier
                    .size(listItemIconSize)
                    .clip(RoundedCornerShape(paddingSmall)),
            )
        },
        title = {
            Text(
                text = model.name,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        },
        trailing = { CountChevron(model.collectionSize) },
    )
}

@Composable
private fun CountChevron(count: Int?) {
    if (count != null) {
        Text(
            text = count.toString(),
            color = MaterialTheme.colorScheme.secondary,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
    ChevronIcon()
}
