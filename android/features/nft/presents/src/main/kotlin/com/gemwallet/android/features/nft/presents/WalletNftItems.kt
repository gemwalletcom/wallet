package com.gemwallet.android.features.nft.presents

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.cases.nft.NftError
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.nft.presents.components.NFTItem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.theme.paddingSmall

private const val NftColumns = 2

fun LazyListScope.walletNftItems(
    items: List<NftItemUIModel>,
    error: NftError?,
    unverifiedCount: Int,
    header: @Composable () -> Unit,
    onAction: (NftListAction) -> Unit,
) {
    if (error != null) {
        item {
            Column(modifier = Modifier.fillParentMaxSize()) {
                header()
                Column(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxWidth()
                        .padding(paddingSmall),
                    verticalArrangement = Arrangement.Center,
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Text(
                        textAlign = TextAlign.Center,
                        text = when (error) {
                            NftError.LoadError -> stringResource(R.string.errors_error_occurred)
                            NftError.NotFoundAsset -> error.message.orEmpty()
                            NftError.NotFoundCollection -> error.message.orEmpty()
                        },
                    )
                    TextButton(onClick = { onAction(NftListAction.Refresh) }) {
                        Text(stringResource(R.string.common_try_again))
                    }
                }
            }
        }
        return
    }

    val showUnverified = unverifiedCount > 0
    if (items.isEmpty() && !showUnverified) {
        item {
            Column(modifier = Modifier.fillParentMaxSize()) {
                header()
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxWidth(),
                    contentAlignment = Alignment.Center,
                ) {
                    EmptyContentView(
                        type = EmptyContentType.Nft(onReceive = { onAction(NftListAction.Receive) }),
                    )
                }
            }
        }
        return
    }

    item { header() }
    items(items.chunked(NftColumns)) { row ->
        Row(
            modifier = Modifier
                .fillParentMaxWidth()
                .padding(horizontal = paddingSmall),
            horizontalArrangement = Arrangement.spacedBy(paddingSmall),
        ) {
            row.forEach { nft ->
                Box(modifier = Modifier.weight(1f)) {
                    NFTItem(
                        model = nft,
                        onClick = {
                            val asset = nft.asset
                            if (asset == null) {
                                onAction(NftListAction.OpenCollection(nft.collection.id.toIdentifier()))
                            } else {
                                onAction(NftListAction.OpenAsset(asset.id))
                            }
                        },
                    )
                }
            }
            if (row.size == 1) {
                Spacer(modifier = Modifier.weight(1f))
            }
        }
    }

    if (showUnverified) {
        item {
            LinkItem(
                title = stringResource(R.string.asset_verification_unverified),
                listPosition = ListPosition.Single,
                trailingContent = {
                    PropertyDataText(
                        text = unverifiedCount.toString(),
                        badge = { DataBadgeChevron() },
                    )
                },
                onClick = { onAction(NftListAction.OpenUnverified) },
            )
        }
    }
}
