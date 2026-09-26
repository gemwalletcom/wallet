package com.gemwallet.android.features.transfer.presents.receive

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
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
import com.gemwallet.android.domains.asset.subtitleSymbol
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.transfer.presents.receive.components.rememberQRCodePainter
import com.gemwallet.android.features.transfer.viewmodels.receive.ReceiveViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.components.clipboard.setCopy
import com.gemwallet.android.ui.components.list_head.CenteredListHead
import com.gemwallet.android.ui.components.list_head.HeaderIcon
import com.gemwallet.android.ui.components.list_item.ChainItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.parseMarkdownToAnnotatedString
import com.gemwallet.android.ui.components.screen.LoadingScene
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.shareText
import com.gemwallet.android.ui.theme.WindowDimension
import com.gemwallet.android.ui.theme.isCompactDimension
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingHalfSmall
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space0
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemLocalizedText

private val qrCardElevation = 3.dp

private val qrSize = 300.dp
private val qrSizeCompact = 220.dp
private val qrMinSize = 100.dp

@Composable
fun ReceiveScreen(assetId: AssetId, closeIcon: Boolean = false, onCancel: () -> Unit) {
    val viewModel = hiltViewModel<ReceiveViewModel, ReceiveViewModel.Factory>(
        key = assetId.toIdentifier(),
    ) { it.create(assetId) }
    val assetInfo by viewModel.asset.collectAsStateWithLifecycle()
    val networks by viewModel.networks.collectAsStateWithLifecycle()
    var isShowingNetworkSelector by remember { mutableStateOf(false) }
    val info = assetInfo

    if (info != null) {
        LaunchedEffect(info.asset.id) {
            viewModel.setVisible()
        }
        ReceiveScene(
            closeIcon = closeIcon,
            assetInfo = info,
            warning = remember(info.asset.id) { viewModel.warningText(info.asset) },
            shareText = viewModel.shareAddress(),
            copyText = viewModel.copyAddress(),
            standard = networks.networks.firstOrNull { it.assetId == info.asset.id.toIdentifier() }?.standard,
            onSelectNetwork = if (networks.showsSelector) {
                { isShowingNetworkSelector = true }
            } else {
                null
            },
            onCancel = onCancel,
        )
        ReceiveNetworkSelector(
            isVisible = isShowingNetworkSelector,
            networks = networks.networks,
            onSelect = viewModel::selectAsset,
            onDismiss = { isShowingNetworkSelector = false },
        )
    } else {
        LoadingScene(title = stringResource(R.string.wallet_receive), onCancel)
    }
}

@Composable
private fun ReceiveScene(closeIcon: Boolean, assetInfo: AssetData, warning: String, shareText: String?, copyText: GemCopy?, standard: GemLocalizedText?, onSelectNetwork: (() -> Unit)?, onCancel: () -> Unit) {
    val context = LocalContext.current
    val clipboardManager = LocalContext.current.clipboardManager()
    val shareTitle = stringResource(R.string.common_share)
    val isCompactHeight = isCompactDimension(WindowDimension.Height)
    val imageSize = if (isCompactHeight) qrSizeCompact else qrSize
    val imagePadding = if (isCompactHeight) paddingSmall else paddingDefault

    val onShare = fun () {
        context.shareText(subject = null, text = shareText, chooserTitle = shareTitle)
    }

    val onCopyClick = fun () {
        copyText?.let { clipboardManager.setCopy(context, it) }
    }

    Scene(
        title = stringResource(R.string.wallet_receive),
        onClose = onCancel,
        closeIcon = closeIcon,
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
                        subtitle = standard?.string(context),
                        listPosition = ListPosition.Single,
                        paddingHorizontal = space0,
                        trailing = { DataBadgeChevron() },
                        onClick = it,
                    )
                    Spacer(modifier = Modifier.size(paddingDefault))
                }
                MainActionButton(onClick = onCopyClick) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(paddingHalfSmall),
                    ) {
                        Icon(AppIcons.ContentCopy, "copy")
                        Text(stringResource(R.string.common_copy))
                    }
                }
            }
        },
    ) {
        if (assetInfo.account.address.isEmpty()) {
            return@Scene
        }
        Spacer(modifier = Modifier.weight(1f))
        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(imagePadding),
        ) {
            CenteredListHead(
                title = assetInfo.asset.name,
                subtitle = assetInfo.asset.subtitleSymbol,
                bottomPadding = space0,
                leading = { HeaderIcon(assetInfo.asset) },
            )
            ElevatedCard(
                modifier = Modifier.width(imageSize),
                elevation = CardDefaults.cardElevation(defaultElevation = qrCardElevation),
                colors = CardDefaults.cardColors(
                    containerColor = Color.White,
                    contentColor = Color.White,
                ),
            ) {
                Box(
                    modifier = Modifier
                        .widthIn(qrMinSize, imageSize)
                        .aspectRatio(1f)
                        .padding(imagePadding)
                        .clickable(onCopyClick),
                ) {
                    rememberQRCodePainter(
                        content = assetInfo.account.address,
                        size = qrSize,
                    )?.let { painter ->
                        Image(
                            modifier = Modifier.fillMaxSize(),
                            painter = painter,
                            contentDescription = null,
                            contentScale = ContentScale.FillWidth,
                        )
                    }
                }
                Text(
                    modifier = Modifier
                        .width(imageSize)
                        .padding(horizontal = imagePadding)
                        .clickable(onCopyClick),
                    text = assetInfo.account.address,
                    textAlign = TextAlign.Center,
                    color = MaterialTheme.colorScheme.secondary,
                    fontWeight = FontWeight.Medium,
                    style = MaterialTheme.typography.bodyMedium,
                )
                Spacer(modifier = Modifier.size(imagePadding))
            }
            Text(
                modifier = Modifier.width(imageSize),
                text = remember(warning) { parseMarkdownToAnnotatedString(warning) },
                textAlign = TextAlign.Center,
                color = MaterialTheme.colorScheme.secondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
        Spacer(modifier = Modifier.weight(1f))
    }
}
