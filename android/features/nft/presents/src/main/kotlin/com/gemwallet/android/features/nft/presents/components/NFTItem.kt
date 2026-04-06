package com.gemwallet.android.features.nft.presents.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import com.gemwallet.android.ui.components.image.NftImage
import com.gemwallet.android.ui.components.image.toImageSource
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.models.actions.NftAssetIdAction
import com.gemwallet.android.ui.models.actions.NftCollectionIdAction
import com.gemwallet.android.ui.theme.emptyImageColor
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space24
import com.gemwallet.android.ui.theme.space6
import com.gemwallet.android.ui.theme.space8

@Composable
fun NFTItem(
    model: NftItemUIModel,
    collectionIdAction: NftCollectionIdAction,
    assetIdAction: NftAssetIdAction,
) {
    Card(
        modifier = Modifier
            .clickable(onClick = { model.onClick(collectionIdAction, assetIdAction) })
            .padding(start = paddingSmall, bottom = paddingDefault, end = paddingSmall),
        colors = CardDefaults.cardColors().copy(containerColor = MaterialTheme.colorScheme.surface),
    ) {
        Column {
            Box {
                NftImage(
                    source = model.toImageSource(),
                    modifier = Modifier
                        .aspectRatio(1f)
                        .clip(RoundedCornerShape(paddingDefault)),
                )
                val count = model.collectionSize
                if (count != null) {
                    CountBadge(
                        count = count,
                        modifier = Modifier
                            .align(Alignment.TopEnd)
                            .padding(space8),
                    )
                }
            }
            NftTitle(
                name = model.name,
                status = model.collection.status,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = paddingSmall),
            )
        }
    }
}

@Composable
private fun CountBadge(count: Int, modifier: Modifier = Modifier) {
    Box(
        contentAlignment = Alignment.Center,
        modifier = modifier
            .defaultMinSize(minWidth = space24, minHeight = space24)
            .background(emptyImageColor, RoundedCornerShape(space8))
            .padding(horizontal = space6),
    ) {
        Text(
            text = count.toString(),
            color = Color.White,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.SemiBold,
        )
    }
}
