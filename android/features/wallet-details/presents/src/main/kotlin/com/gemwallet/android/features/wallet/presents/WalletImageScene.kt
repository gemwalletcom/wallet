package com.gemwallet.android.features.wallet.presents

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.PrimaryTabRow
import androidx.compose.material3.Tab
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import com.gemwallet.android.domains.wallet.aggregates.WalletDetailsAggregate
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.NftImage
import com.gemwallet.android.ui.components.image.WalletAvatar
import com.gemwallet.android.ui.components.image.toImageSource
import com.gemwallet.android.ui.components.list_item.supportIcon
import com.gemwallet.android.ui.components.list_item.walletItemIconModel
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.NftItemUIModel
import com.gemwallet.android.ui.theme.Spacer16
import com.gemwallet.android.ui.theme.extraLargeIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall

private const val EMOJI_COLUMNS = 4
private const val NFT_COLUMNS = 2
private const val EMOJI_SCALE = 0.45f

private enum class WalletImageTab { EMOJI, COLLECTIONS }

@Composable
internal fun WalletImageScene(
    wallet: WalletDetailsAggregate?,
    emojis: List<String>,
    nftImages: List<NftItemUIModel>,
    source: WalletImageSource,
    onAction: (WalletImageAction) -> Unit,
) {
    wallet ?: return
    var selectedTab by remember { mutableStateOf(WalletImageTab.EMOJI) }
    val emojiBackground = MaterialTheme.colorScheme.surface.toArgb()
    val onEmoji: (String) -> Unit = { onAction(WalletImageAction.SetEmoji(it, emojiBackground)) }

    Scene(
        title = stringResource(id = R.string.common_avatar),
        onClose = { onAction(WalletImageAction.Close) },
    ) {
        Column(
            modifier = Modifier.fillMaxSize(),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            WalletAvatar(
                imageUrl = wallet.imageUrl,
                placeholder = walletItemIconModel(wallet.type, wallet.walletChain),
                size = extraLargeIconSize,
                modifier = Modifier.padding(top = paddingDefault),
                supportIcon = wallet.type.supportIcon(),
                onClick = { onAction(WalletImageAction.ResetToDefault) },
            )
            Spacer16()
            when (source) {
                WalletImageSource.Onboarding -> Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .weight(1f),
                ) {
                    EmojiGrid(emojis = emojis, onEmoji = onEmoji)
                }

                WalletImageSource.Wallet -> {
                    PrimaryTabRow(
                        selectedTabIndex = selectedTab.ordinal,
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        Tab(
                            selected = selectedTab == WalletImageTab.EMOJI,
                            onClick = { selectedTab = WalletImageTab.EMOJI },
                            text = { Text(stringResource(id = R.string.common_emoji)) },
                        )
                        Tab(
                            selected = selectedTab == WalletImageTab.COLLECTIONS,
                            onClick = { selectedTab = WalletImageTab.COLLECTIONS },
                            text = { Text(stringResource(id = R.string.nft_collections)) },
                        )
                    }
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .weight(1f),
                    ) {
                        when (selectedTab) {
                            WalletImageTab.EMOJI -> EmojiGrid(emojis = emojis, onEmoji = onEmoji)
                            WalletImageTab.COLLECTIONS -> if (nftImages.isEmpty()) {
                                Text(
                                    text = stringResource(id = R.string.nft_state_empty_title),
                                    color = MaterialTheme.colorScheme.secondary,
                                    textAlign = TextAlign.Center,
                                    modifier = Modifier
                                        .align(Alignment.Center)
                                        .padding(paddingDefault),
                                )
                            } else {
                                NftGrid(
                                    nftImages = nftImages,
                                    onNftImage = { onAction(WalletImageAction.SetNftImage(it)) },
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun EmojiGrid(
    emojis: List<String>,
    onEmoji: (String) -> Unit,
) = PickerGrid(columns = EMOJI_COLUMNS, entries = emojis) { emoji ->
    BoxWithConstraints(
        modifier = Modifier
            .fillMaxSize()
            .clip(CircleShape)
            .background(MaterialTheme.colorScheme.surface)
            .clickable { onEmoji(emoji) },
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = emoji,
            fontSize = with(LocalDensity.current) { (maxWidth * EMOJI_SCALE).toSp() },
        )
    }
}

@Composable
private fun NftGrid(
    nftImages: List<NftItemUIModel>,
    onNftImage: (String) -> Unit,
) = PickerGrid(columns = NFT_COLUMNS, entries = nftImages) { item ->
    val source = item.toImageSource()
    Box(
        modifier = Modifier
            .fillMaxSize()
            .clip(RoundedCornerShape(paddingDefault))
            .clickable { onNftImage(source.url) },
    ) {
        NftImage(
            source = source,
            modifier = Modifier.fillMaxSize(),
        )
    }
}

@Composable
private fun <T> PickerGrid(
    columns: Int,
    entries: List<T>,
    cell: @Composable (T) -> Unit,
) {
    LazyVerticalGrid(
        columns = GridCells.Fixed(columns),
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(paddingDefault),
    ) {
        items(entries) { entry ->
            Box(
                modifier = Modifier
                    .aspectRatio(1f)
                    .padding(paddingSmall),
            ) {
                cell(entry)
            }
        }
    }
}
