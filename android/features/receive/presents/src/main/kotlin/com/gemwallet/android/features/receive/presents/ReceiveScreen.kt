package com.gemwallet.android.features.receive.presents

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.networkFullName
import com.gemwallet.android.ext.boldMarkdown
import uniffi.gemstone.GemMemoWarning
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.features.receive.presents.components.rememberQRCodePainter
import com.gemwallet.android.features.receive.viewmodels.ReceiveViewModel
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.parseMarkdownToAnnotatedString
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.subtitleSymbol
import com.gemwallet.android.ui.shareText
import com.gemwallet.android.ui.theme.WindowDimension
import com.gemwallet.android.ui.theme.isCompactDimension
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import com.gemwallet.android.ui.components.clipboard.clipboardManager

private val qrSize = 300.dp
private val qrSizeCompact = 220.dp
private val qrMinSize = 100.dp

@Composable
fun ReceiveScreen(onCancel: () -> Unit) {
    val viewModel: ReceiveViewModel = hiltViewModel()
    val assetInfo by viewModel.asset.collectAsStateWithLifecycle()
    val networkAssetIds by viewModel.networkAssetIds.collectAsStateWithLifecycle()
    var isShowingNetworkSelector by remember { mutableStateOf(false) }
    val info = assetInfo

    if (info != null) {
        LaunchedEffect(info.asset.id) {
            viewModel.setVisible()
        }
        ReceiveScene(
            assetInfo = info,
            memoWarning = viewModel.memoWarning(info.asset.id.chain),
            onSelectNetwork = if (networkAssetIds.size > 1) {
                { isShowingNetworkSelector = true }
            } else {
                null
            },
            onCancel = onCancel,
        )
        ReceiveNetworkSelector(
            isVisible = isShowingNetworkSelector,
            assetIds = networkAssetIds,
            onSelect = viewModel::selectAsset,
            onDismiss = { isShowingNetworkSelector = false },
        )
    } else {
        LoadingScene(title = stringResource(R.string.wallet_receive), onCancel)
    }
}

@Composable
private fun ReceiveScene(
    assetInfo: AssetInfo,
    memoWarning: GemMemoWarning,
    onSelectNetwork: (() -> Unit)?,
    onCancel: () -> Unit,
) {
    val context = LocalContext.current
    val clipboardManager = LocalContext.current.clipboardManager()
    val shareTitle = stringResource(R.string.common_share)
    val isCompactHeight = isCompactDimension(WindowDimension.Height)
    val imageSize = if (isCompactHeight) qrSizeCompact else qrSize
    val imagePadding = if (isCompactHeight) paddingSmall else paddingDefault

    val onShare = fun () {
        val subject = "${assetInfo.owner?.chain}\n${assetInfo.asset.symbol}"
        context.shareText(subject = subject, text = assetInfo.owner?.address, chooserTitle = shareTitle)
    }

    val onCopyClick = fun () {
        clipboardManager.setPlainText(context, assetInfo.owner?.address ?: "")
    }

    Scene(
        title = stringResource(R.string.receive_title, ""),
        onClose = onCancel,
        actions = {
            IconButton(onShare) {
                Icon(AppIcons.Share, "")
            }
        },
        mainAction = {
            Column {
                onSelectNetwork?.let {
                    ChainItem(
                        title = assetInfo.asset.id.chain.networkName(),
                        icon = assetInfo.asset.id.chain,
                        subtitle = assetInfo.asset.type.string,
                        listPosition = ListPosition.Single,
                        paddingHorizontal = 0.dp,
                        trailing = { DataBadgeChevron() },
                        onClick = it,
                    )
                    Spacer(modifier = Modifier.size(paddingDefault))
                }
                MainActionButton(onClick = onCopyClick) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(paddingHalfSmall)
                    ) {
                        Icon(AppIcons.ContentCopy, "copy")
                        Text(stringResource(R.string.common_copy))
                    }
                }
            }
        }
    ) {
        if (assetInfo.owner?.address.isNullOrEmpty()) {
            return@Scene
        }
        Spacer(modifier = Modifier.weight(1f))
        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(imagePadding)
        ) {
            CenteredListHead(
                title = assetInfo.asset.name,
                subtitle = assetInfo.asset.subtitleSymbol,
                bottomPadding = 0.dp,
                leading = { HeaderIcon(assetInfo.asset) },
            )
            ElevatedCard(
                modifier = Modifier.width(imageSize),
                elevation = CardDefaults.cardElevation(defaultElevation = 3.dp),
                colors = CardDefaults.cardColors(
                    containerColor = Color.White,
                    contentColor = Color.White,
                )
            ) {
                Image(
                    modifier = Modifier
                        .widthIn(qrMinSize, imageSize)
                        .heightIn(qrMinSize, imageSize)
                        .padding(imagePadding)
                        .clickable(onCopyClick),
                    painter = rememberQRCodePainter(
                        content = assetInfo.owner?.address ?: "",
                        cacheName = "${assetInfo.owner?.chain?.string}_${assetInfo.owner?.address}",
                        size = qrSize
                    ),
                    contentDescription = null,
                    contentScale = ContentScale.FillWidth
                )
                Text(
                    modifier = Modifier
                        .width(imageSize)
                        .padding(horizontal = imagePadding)
                        .clickable(onCopyClick),
                    text = assetInfo.owner?.address ?: "",
                    textAlign = TextAlign.Center,
                    color = MaterialTheme.colorScheme.secondary,
                    fontWeight = FontWeight.Medium,
                    style = MaterialTheme.typography.bodyMedium,
                )
                Spacer(modifier = Modifier.size(imagePadding))
            }
            Text(
                modifier = Modifier.width(imageSize),
                text = parseMarkdownToAnnotatedString(warningMessage(assetInfo.asset, memoWarning)),
                textAlign = TextAlign.Center,
                color = MaterialTheme.colorScheme.secondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
        Spacer(modifier = Modifier.weight(1f))
    }
}

@Composable
private fun warningMessage(asset: Asset, memoWarning: GemMemoWarning): String {
    val warning = stringResource(
        R.string.receive_warning,
        asset.symbol.boldMarkdown(),
        asset.networkFullName.boldMarkdown(),
    )
    val memoText = when (memoWarning) {
        GemMemoWarning.DESTINATION_TAG -> stringResource(R.string.wallet_receive_no_destination_tag_required)
        GemMemoWarning.MEMO -> stringResource(R.string.wallet_receive_no_memo_required)
        GemMemoWarning.NOT_SUPPORTED -> null
    }
    return listOfNotNull(warning, memoText).joinToString(" ")
}
